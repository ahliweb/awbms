// VG-01 frozen-corpus consumer: reads the frozen AWCMS / AWCMS-Astro evidence
// and fails when fixtures contradict each other, when a frozen file has no
// consumer, or when the fixture-usage map names a consumer that does not exist.
//
// std-only. Build and run (this is what tools/vg01_verify_frozen.sh does):
//   rustc --edition 2024 -D warnings tools/vg01_check_corpus.rs -o /tmp/vg01_check_corpus
//   /tmp/vg01_check_corpus contracts/legacy/awcms [repo-root]
//
// Scope limit: this tool never regenerates anything and never reads AWCMS.
// Checks that need the pinned AWCMS checkout (per-file migration SHA-256
// against real SQL, live DB introspection) belong to the vg01-freeze workflow.

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const THIS_TOOL: &str = "tools/vg01_check_corpus.rs";

// ---------------------------------------------------------------- JSON ----

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(items) => items.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    fn str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    fn arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }
    fn num(&self) -> Option<u64> {
        match self {
            Json::Num(n) if *n >= 0.0 && n.fract() == 0.0 => Some(*n as u64),
            _ => None,
        }
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn eat(&mut self, c: u8) -> Result<(), String> {
        self.ws();
        if self.b.get(self.i) == Some(&c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("json: expected '{}' at byte {}", c as char, self.i))
        }
    }
    fn lit(&mut self, word: &str, v: Json) -> Result<Json, String> {
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(format!("json: bad literal at byte {}", self.i))
        }
    }
    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.b.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(Json::Obj(items));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.eat(b':')?;
                    items.push((k, self.value()?));
                    self.ws();
                    match self.b.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Json::Obj(items));
                        }
                        _ => return Err(format!("json: bad object at byte {}", self.i)),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(Json::Arr(items));
                }
                loop {
                    items.push(self.value()?);
                    self.ws();
                    match self.b.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Json::Arr(items));
                        }
                        _ => return Err(format!("json: bad array at byte {}", self.i)),
                    }
                }
            }
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') => self.lit("true", Json::Bool(true)),
            Some(b'f') => self.lit("false", Json::Bool(false)),
            Some(b'n') => self.lit("null", Json::Null),
            Some(_) => {
                let start = self.i;
                while self.i < self.b.len()
                    && matches!(self.b[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                {
                    self.i += 1;
                }
                std::str::from_utf8(&self.b[start..self.i])
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(Json::Num)
                    .ok_or_else(|| format!("json: bad value at byte {start}"))
            }
            None => Err("json: unexpected end".into()),
        }
    }
    fn string(&mut self) -> Result<String, String> {
        if self.b.get(self.i) != Some(&b'"') {
            return Err(format!("json: expected string at byte {}", self.i));
        }
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        while let Some(&c) = self.b.get(self.i) {
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(out).map_err(|e| e.to_string()),
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or("json: bad escape")?;
                    self.i += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' | b'f' => {}
                        b'u' => {
                            let hex = self.b.get(self.i..self.i + 4).ok_or("json: bad \\u")?;
                            let cp = u32::from_str_radix(
                                std::str::from_utf8(hex).map_err(|e| e.to_string())?,
                                16,
                            )
                            .map_err(|e| e.to_string())?;
                            self.i += 4;
                            let ch = char::from_u32(cp).unwrap_or('\u{fffd}');
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        other => out.push(other),
                    }
                }
                other => out.push(other),
            }
        }
        Err("json: unterminated string".into())
    }
}

fn parse_json(text: &str) -> Result<Json, String> {
    let mut p = Parser { b: text.as_bytes(), i: 0 };
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err(format!("json: trailing data at byte {}", p.i));
    }
    Ok(v)
}

// ------------------------------------------------------------------ CSV ----

fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => row.push(std::mem::take(&mut field)),
            ('\n', false) => {
                row.push(std::mem::take(&mut field));
                rows.push(std::mem::take(&mut row));
            }
            ('\r', false) => {}
            (c, _) => field.push(c),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

// -------------------------------------------------------------- context ----

struct Ctx {
    root: PathBuf,
    repo: PathBuf,
    failures: Vec<String>,
    ran: BTreeSet<String>,
    notes: RefCell<Vec<String>>,
    /// Artifacts listed in fixture-usage.json `pendingArtifacts`.
    pending: BTreeSet<String>,
}

type R = Result<(), String>;

fn ensure(cond: bool, msg: impl Into<String>) -> R {
    if cond { Ok(()) } else { Err(msg.into()) }
}

fn ensure_eq<T: PartialEq + std::fmt::Debug>(label: &str, a: T, b: T) -> R {
    if a == b { Ok(()) } else { Err(format!("{label}: {a:?} != {b:?}")) }
}

impl Ctx {
    fn read(&self, rel: &str) -> Result<String, String> {
        fs::read_to_string(self.root.join(rel)).map_err(|e| format!("{rel}: {e}"))
    }
    fn json(&self, rel: &str) -> Result<Json, String> {
        parse_json(&self.read(rel)?).map_err(|e| format!("{rel}: {e}"))
    }
    fn repo_read(&self, rel: &str) -> Result<String, String> {
        fs::read_to_string(self.repo.join(rel)).map_err(|e| format!("{rel}: {e}"))
    }
    fn lines(&self, rel: &str) -> Result<Vec<String>, String> {
        Ok(self.read(rel)?.lines().map(str::to_owned).collect())
    }
    fn check(&mut self, name: &str, f: impl FnOnce(&Ctx) -> R) {
        self.ran.insert(name.to_owned());
        match f(self) {
            Ok(()) => println!("ok    check:{name}"),
            Err(e) => {
                println!("FAIL  check:{name}: {e}");
                self.failures.push(format!("{name}: {e}"));
            }
        }
    }
}

const S: &str = "frozen/static";
const SRC: &str = "frozen/static/source";
const L: &str = "frozen/live";
const INV: &str = "frozen/static/source/awcms-repo-inventory.md";
const VECTORS: &str = "authorization/vectors.json";

fn is_sha1_hex(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

struct Provenance {
    awcms: String,
    astro: String,
}

fn provenance(c: &Ctx) -> Result<Provenance, String> {
    let j = c.json("frozen/provenance.json")?;
    let get = |k: &str| {
        j.get(k)
            .and_then(Json::str)
            .map(str::to_owned)
            .ok_or(format!("provenance.json: missing {k}"))
    };
    let p = Provenance { awcms: get("awcms")?, astro: get("awcmsAstro")? };
    ensure(is_sha1_hex(&p.awcms) && is_sha1_hex(&p.astro), "provenance SHAs are not 40-hex")?;
    Ok(p)
}

// ----------------------------------------------------- inventory.md parse ----

fn md_section<'a>(text: &'a str, heading: &str, next: &str) -> Result<&'a str, String> {
    let start = text.find(heading).ok_or(format!("inventory: missing {heading}"))?;
    let rest = &text[start + heading.len()..];
    let end = rest.find(next).ok_or(format!("inventory: missing {next}"))?;
    Ok(&rest[..end])
}

fn md_rows(section: &str) -> Vec<Vec<String>> {
    section
        .lines()
        .filter(|l| l.starts_with('|') && !l.contains("---"))
        .skip(1) // header row
        .map(|l| {
            l.trim_matches('|')
                .split('|')
                .map(|c| c.trim().trim_matches('`').to_owned())
                .collect()
        })
        .collect()
}

struct Inventory {
    summary: BTreeMap<String, u64>,
    modules: Vec<String>,
    migrations: Vec<String>,
    tables: Vec<(String, String, bool, bool)>,
    tests: u64,
    routes: BTreeMap<String, u64>,
}

fn load_inventory(c: &Ctx) -> Result<Inventory, String> {
    let t = c.read(INV)?;
    let num = |s: &str| s.parse::<u64>().map_err(|e| format!("inventory number '{s}': {e}"));
    let mut summary = BTreeMap::new();
    for r in md_rows(md_section(&t, "### Summary", "### Modules")?) {
        summary.insert(r[0].clone(), num(&r[1])?);
    }
    let modules = md_rows(md_section(&t, "### Modules", "### Migrations")?)
        .into_iter()
        .map(|r| r[0].clone())
        .collect();
    let migrations = md_rows(md_section(&t, "### Migrations", "### Tables")?)
        .into_iter()
        .map(|r| r[1].clone())
        .collect();
    let tables = md_rows(md_section(&t, "### Tables & Row-Level Security", "### Tests")?)
        .into_iter()
        .map(|r| (r[0].clone(), r[1].clone(), r[2] == "yes", r[3] == "yes"))
        .collect();
    let mut tests = 0;
    for r in md_rows(md_section(&t, "### Tests", "### Routes")?) {
        tests += num(&r[1])?;
    }
    let mut routes = BTreeMap::new();
    for r in md_rows(md_section(&t, "### Routes", "<!-- END GENERATED")?) {
        routes.insert(r[0].clone(), num(&r[1])?);
    }
    Ok(Inventory { summary, modules, migrations, tables, tests, routes })
}

// ------------------------------------------------------------ TS parsing ----

fn ts_record_keys(text: &str, start: &str, end: &str) -> Result<Vec<String>, String> {
    let s = text.find(start).ok_or(format!("ts: missing {start}"))?;
    let rest = &text[s..];
    let e = rest[start.len()..].find(end).ok_or(format!("ts: missing {end}"))? + start.len();
    Ok(rest[..e]
        .lines()
        .filter_map(|l| {
            let l = l.trim_end();
            if l.starts_with("  \"/api/") && l.ends_with("\":") {
                Some(l.trim().trim_end_matches(':').trim_matches('"').to_owned())
            } else {
                None
            }
        })
        .collect())
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

// ---------------------------------------------------------------- checks ----

fn check_stale_sha(c: &Ctx) -> R {
    let p = provenance(c)?;
    let by_repo = |repo: &str| match repo {
        "ahliweb/awcms" => Some(&p.awcms),
        "ahliweb/awcms-astro" => Some(&p.astro),
        _ => None,
    };
    for wf in [".github/workflows/verification.yml", ".github/workflows/vg01-freeze.yml"] {
        let text = c.repo_read(wf)?;
        let mut repo = String::new();
        let mut seen: BTreeSet<&'static str> = BTreeSet::new();
        for line in text.lines() {
            let l = line.trim();
            if let Some(r) = l.strip_prefix("repository:") {
                repo = r.trim().to_owned();
            } else if let Some(r) = l.strip_prefix("ref:") {
                if let Some(want) = by_repo(&repo) {
                    let got = r.trim();
                    ensure(
                        got == want,
                        format!("{wf}: checkout of {repo} pinned to {got}, provenance.json says {want}"),
                    )?;
                    seen.insert(if repo.ends_with("-astro") { "astro" } else { "awcms" });
                }
            } else if let Some(r) = l.strip_prefix("\"awcms\":") {
                let got = r.trim().trim_matches(|ch| ch == ',' || ch == '"');
                ensure(got == p.awcms, format!("{wf}: provenance heredoc awcms {got} != {}", p.awcms))?;
            } else if let Some(r) = l.strip_prefix("\"awcmsAstro\":") {
                let got = r.trim().trim_matches(|ch| ch == ',' || ch == '"');
                ensure(got == p.astro, format!("{wf}: provenance heredoc awcmsAstro {got} != {}", p.astro))?;
            }
        }
        ensure(
            seen.contains("awcms") && seen.contains("astro"),
            format!("{wf}: both AWCMS and AWCMS-Astro checkouts must be SHA-pinned"),
        )?;
    }
    // Every other place that records a source SHA must agree with provenance.
    let tool = c.repo_read("tools/vg01_inventory.rs")?;
    for (key, want) in [("AWCMS_SHA", &p.awcms), ("ASTRO_SHA", &p.astro)] {
        let line = tool
            .lines()
            .find(|l| l.starts_with(&format!("const {key}:")))
            .ok_or(format!("tools/vg01_inventory.rs: missing {key}"))?;
        ensure(line.contains(want.as_str()), format!("tools/vg01_inventory.rs {key} is stale"))?;
    }
    let pair = |j: &Json, a: &str, b: &str, what: &str| -> R {
        ensure_eq(&format!("{what} awcms"), j.get(a).and_then(Json::str), Some(p.awcms.as_str()))?;
        ensure_eq(&format!("{what} astro"), j.get(b).and_then(Json::str), Some(p.astro.as_str()))
    };
    pair(&c.json(&format!("{S}/astro-consumer.json"))?, "awcms_source_sha", "astro_source_sha", "astro-consumer.json")?;
    for f in ["modules.json", "migrations.json"] {
        let j = c.json(&format!("{S}/{f}"))?;
        ensure_eq(f, j.get("source_sha").and_then(Json::str), Some(p.awcms.as_str()))?;
    }
    ensure_eq(
        "live/summary.json source_commit",
        c.json(&format!("{L}/summary.json"))?.get("source_commit").and_then(Json::str).map(str::to_owned),
        Some(p.awcms.clone()),
    )?;
    let commits = |j: &Json, what: &str| -> R {
        let got: Vec<&str> = j
            .get("sources")
            .and_then(Json::arr)
            .ok_or(format!("{what}: no sources"))?
            .iter()
            .filter_map(|s| s.get("commit").and_then(Json::str))
            .collect();
        ensure_eq(what, got, vec![p.awcms.as_str(), p.astro.as_str()])
    };
    commits(&c.json(&format!("{S}/manifest.json"))?, "static/manifest.json sources")?;
    commits(&c.json("manifest.json")?, "manifest.json sources")?;
    let v = c.json(VECTORS)?;
    let src = v.get("sources").ok_or("vectors.json: no sources")?;
    ensure_eq("vectors awcms", src.get("awcms").and_then(Json::str), Some(p.awcms.as_str()))?;
    ensure_eq("vectors awcmsAstro", src.get("awcmsAstro").and_then(Json::str), Some(p.astro.as_str()))
}

fn check_modules(c: &Ctx) -> R {
    let reg = c.read(&format!("{SRC}/awcms-module-registry.ts"))?;
    let mut ident_dir = BTreeMap::new();
    for l in reg.lines() {
        let l = l.trim();
        if l.starts_with("import {") && l.ends_with("/module\";") {
            let ident = l["import {".len()..].split('}').next().unwrap_or("").trim().to_owned();
            let from = l.split("from \"./").nth(1).unwrap_or("");
            ident_dir.insert(ident, from.trim_end_matches("/module\";").to_owned());
        }
    }
    let start = reg.find("const baseModules: ModuleDescriptor[] = [").ok_or("registry: no baseModules")?;
    let body = &reg[start..];
    let body = &body[..body.find("\n];").ok_or("registry: baseModules unterminated")?];
    let mut from_array = Vec::new();
    for l in body.lines().skip(1) {
        let l = l.trim().trim_end_matches(',');
        if l.ends_with("Module") && !l.starts_with("//") && !l.starts_with('*') {
            from_array.push(ident_dir.get(l).cloned().ok_or(format!("registry: {l} has no import"))?);
        }
    }
    let from_array = sorted(from_array);
    let j = c.json(&format!("{S}/modules.json"))?;
    let listed: Vec<String> = j
        .get("modules")
        .and_then(Json::arr)
        .ok_or("modules.json: no modules")?
        .iter()
        .filter_map(|m| m.str().map(str::to_owned))
        .collect();
    ensure_eq("modules.json count field", j.get("count").and_then(Json::num), Some(listed.len() as u64))?;
    ensure_eq("modules.json vs registry baseModules array", sorted(listed.clone()), from_array.clone())?;
    ensure_eq("module count", listed.len(), 24)?;
    // inventory keys are snake_case descriptor keys; one documented alias.
    let inv = load_inventory(c)?;
    let norm: Vec<String> = sorted(
        inv.modules
            .iter()
            .map(|k| k.replace('_', "-"))
            .map(|k| if k == "workflow" { "workflow-approval".to_owned() } else { k })
            .collect(),
    );
    ensure_eq("inventory.md module keys (workflow aliased to workflow-approval)", norm, from_array)?;
    ensure_eq("inventory.md summary modules", inv.summary.get("Registered modules").copied(), Some(24))
}

fn check_migrations(c: &Ctx) -> R {
    let j = c.json(&format!("{S}/migrations.json"))?;
    let items = j.get("migrations").and_then(Json::arr).ok_or("migrations.json: no migrations")?;
    let mut paths = Vec::new();
    let mut digests = BTreeSet::new();
    for (i, m) in items.iter().enumerate() {
        let path = m.get("path").and_then(Json::str).ok_or("migration without path")?;
        let sha = m.get("sha256").and_then(Json::str).ok_or("migration without sha256")?;
        ensure(is_sha256_hex(sha), format!("{path}: sha256 is not 64 lowercase hex"))?;
        ensure(digests.insert(sha.to_owned()), format!("{path}: duplicate sha256"))?;
        let want_prefix = format!("sql/{:03}_", i + 1);
        ensure(path.starts_with(&want_prefix) && path.ends_with(".sql"), format!("{path}: expected {want_prefix}*.sql (contiguous order)"))?;
        paths.push(path.to_owned());
    }
    ensure_eq("migrations.json count field", j.get("count").and_then(Json::num), Some(paths.len() as u64))?;
    ensure_eq("migration count", paths.len(), 148)?;
    let inv = load_inventory(c)?;
    ensure_eq("inventory.md migration list vs migrations.json", &inv.migrations, &paths)?;
    ensure_eq("inventory.md summary migrations", inv.summary.get("Migrations").copied(), Some(148))?;
    let ledger = parse_csv(&c.read(&format!("{L}/migration-ledger.csv"))?);
    let names: Vec<&str> = ledger.iter().skip(1).map(|r| r[0].as_str()).collect();
    let want: Vec<&str> = paths.iter().map(|p| p.trim_start_matches("sql/")).collect();
    ensure_eq("live migration ledger names vs migrations.json (order)", names, want)?;
    for r in ledger.iter().skip(1) {
        ensure(
            r[1].strip_prefix("sha256:").is_some_and(is_sha256_hex),
            format!("ledger {}: checksum is not sha256:<64 hex>", r[0]),
        )?;
    }
    Ok(())
}

fn tsv(c: &Ctx, rel: &str) -> Result<BTreeMap<String, (bool, bool)>, String> {
    let mut m = BTreeMap::new();
    for l in c.lines(rel)? {
        let f: Vec<&str> = l.split('\t').collect();
        ensure(f.len() == 3, format!("{rel}: bad row '{l}'"))?;
        m.insert(f[0].to_owned(), (f[1] == "t", f[2] == "t"));
    }
    Ok(m)
}

fn check_tables_rls(c: &Ctx) -> R {
    let inv = load_inventory(c)?;
    let inv_map: BTreeMap<String, (bool, bool)> =
        inv.tables.iter().map(|t| (t.0.clone(), (t.2, t.3))).collect();
    ensure_eq("inventory.md table row count", inv.tables.len(), inv_map.len())?;
    let migs: BTreeSet<String> = c
        .json(&format!("{S}/migrations.json"))?
        .get("migrations")
        .and_then(Json::arr)
        .ok_or("no migrations")?
        .iter()
        .filter_map(|m| m.get("path").and_then(Json::str).map(str::to_owned))
        .collect();
    for t in &inv.tables {
        ensure(migs.contains(&t.1), format!("table {} created in unknown migration {}", t.0, t.1))?;
        ensure(!t.3 || t.2, format!("table {}: FORCE without RLS", t.0))?;
    }
    let src = tsv(c, &format!("{L}/source-tables-rls.tsv"))?;
    let live = tsv(c, &format!("{L}/live-tables-rls.tsv"))?;
    ensure(src == inv_map, "source-tables-rls.tsv differs from inventory.md table/RLS/FORCE state")?;
    ensure(src == live, "source-tables-rls.tsv differs from live-tables-rls.tsv")?;
    let names: BTreeSet<String> = inv_map.keys().cloned().collect();
    for f in ["source-table-names.txt", "live-table-names.txt"] {
        let n: BTreeSet<String> = c.lines(&format!("{L}/{f}"))?.into_iter().collect();
        ensure(n == names, format!("{f} differs from inventory.md table set"))?;
    }
    let rows = parse_csv(&c.read(&format!("{L}/tables-rls.csv"))?);
    let mut csv = BTreeMap::new();
    let mut policy_sum = 0usize;
    for r in rows.iter().skip(1) {
        let pc: usize = r[5].parse().map_err(|e| format!("policy_count: {e}"))?;
        policy_sum += pc;
        ensure(r[3] != "t" || pc > 0, format!("{}: RLS enabled with zero policies", r[0]))?;
        csv.insert(r[0].clone(), (r[3] == "t", r[4] == "t"));
    }
    ensure(csv == live, "tables-rls.csv (pg_class) differs from live-tables-rls.tsv")?;
    let pol = parse_csv(&c.read(&format!("{L}/policies.csv"))?);
    ensure_eq("policies.csv rows vs sum(policy_count)", pol.len() - 1, policy_sum)?;
    for r in pol.iter().skip(1) {
        ensure(live.get(&r[0]).is_some_and(|s| s.0), format!("policy {} on table without RLS", r[1]))?;
    }
    let rls = inv_map.values().filter(|s| s.0).count();
    let force = inv_map.values().filter(|s| s.1).count();
    ensure_eq("table count", inv_map.len(), 152)?;
    ensure_eq("RLS count", rls, 134)?;
    ensure_eq("FORCE count", force, 134)?;
    ensure_eq("inventory.md summary FORCE", inv.summary.get("Tables with `FORCE` RLS").copied(), Some(force as u64))?;
    ensure_eq("inventory.md summary RLS-free", inv.summary.get("RLS-free tables (global, by design)").copied(), Some((inv_map.len() - rls) as u64))?;
    ensure_eq("inventory.md summary tables", inv.summary.get("awcms_*` tables").copied(), Some(inv_map.len() as u64))
}

fn check_static_vs_live(c: &Ctx) -> R {
    for f in ["source-only-tables.txt", "live-only-tables.txt", "source-vs-live-tables.diff"] {
        ensure(c.read(&format!("{L}/{f}"))?.is_empty(), format!("{f} is non-empty: unexplained static-vs-live drift"))?;
    }
    // The 152 (all awcms_*) vs 151 (business) difference is the migration
    // ledger table, present in both source and live sets, and nothing else.
    let names: BTreeSet<String> = c.lines(&format!("{L}/live-table-names.txt"))?.into_iter().collect();
    ensure(names.contains("awcms_schema_migrations"), "migration ledger table missing from live table set")?;
    let s = c.json(&format!("{L}/summary.json"))?;
    let n = |k: &str| s.get(k).and_then(Json::num).ok_or(format!("summary.json: missing {k}"));
    ensure_eq("all - ledger = business", n("live_all_awcms_table_count")? - n("live_migration_ledger_table_count")?, n("live_business_table_count_excluding_migration_ledger")?)?;
    ensure_eq("non-force all - ledger = non-force business", n("live_non_force_all_awcms_table_count")? - n("live_migration_ledger_table_count")?, n("live_non_force_business_table_count")?)?;
    ensure(
        parse_csv(&c.read(&format!("{L}/tables-rls.csv"))?)
            .iter()
            .any(|r| r[0] == "awcms_schema_migrations" && r[3] == "f"),
        "migration ledger must be an RLS-free table in live introspection",
    )
}

fn check_live_roles(c: &Ctx) -> R {
    let roles = parse_csv(&c.read(&format!("{L}/roles.csv"))?);
    let mut names = BTreeSet::new();
    for r in roles.iter().skip(1) {
        ensure(r[1] == "f" && r[2] == "f", format!("role {} has SUPERUSER or BYPASSRLS", r[0]))?;
        ensure(r[3] == "f" && r[4] == "f", format!("role {} has CREATEROLE or CREATEDB", r[0]))?;
        names.insert(r[0].clone());
    }
    for must in ["awcms_app", "awcms_worker"] {
        ensure(names.contains(must), format!("role {must} missing"))?;
    }
    let tables = parse_csv(&c.read(&format!("{L}/tables-rls.csv"))?);
    let owners: BTreeSet<&str> = tables.iter().skip(1).map(|r| r[2].as_str()).collect();
    ensure_eq("distinct table owners", owners.len(), 1)?;
    ensure(!owners.contains("awcms_app") && !owners.contains("awcms_worker"), "runtime role owns tables")?;
    let tnames: BTreeSet<&str> = tables.iter().skip(1).map(|r| r[0].as_str()).collect();
    let grants = parse_csv(&c.read(&format!("{L}/table-grants.csv"))?);
    for r in grants.iter().skip(1) {
        ensure(names.contains(&r[0]), format!("table grant to unknown role {}", r[0]))?;
        ensure(tnames.contains(r[1].as_str()), format!("table grant on unknown table {}", r[1]))?;
        ensure(r[3] == "NO", format!("grantable privilege {} {}", r[0], r[1]))?;
        ensure(r[0] != "awcms_vg01_owner", "runtime evidence grants to owner")?;
    }
    let rg = parse_csv(&c.read(&format!("{L}/routine-grants.csv"))?);
    for r in rg.iter().skip(1) {
        ensure(names.contains(&r[0]) && r[2] == "EXECUTE", format!("unexpected routine grant {r:?}"))?;
    }
    let s = c.json(&format!("{L}/summary.json"))?;
    ensure_eq("summary role_count", s.get("role_count").and_then(Json::num), Some((roles.len() - 1) as u64))?;
    ensure_eq("summary bad_security_role_count", s.get("bad_security_role_count").and_then(Json::num), Some(0))?;
    ensure_eq("summary runtime_owned_table_count", s.get("runtime_owned_table_count").and_then(Json::num), Some(0))
}

fn check_live_summary(c: &Ctx) -> R {
    let s = c.json(&format!("{L}/summary.json"))?;
    let n = |k: &str| s.get(k).and_then(Json::num).ok_or(format!("summary.json: missing {k}"));
    let inv = load_inventory(c)?;
    let force = inv.tables.iter().filter(|t| t.3).count() as u64;
    let rls = inv.tables.iter().filter(|t| t.2).count() as u64;
    let all = inv.tables.len() as u64;
    let ledger = (parse_csv(&c.read(&format!("{L}/migration-ledger.csv"))?).len() - 1) as u64;
    let pol = (parse_csv(&c.read(&format!("{L}/policies.csv"))?).len() - 1) as u64;
    ensure_eq("migration_count", n("migration_count")?, ledger)?;
    ensure_eq("source_derived_all_awcms_table_count", n("source_derived_all_awcms_table_count")?, all)?;
    ensure_eq("live_all_awcms_table_count", n("live_all_awcms_table_count")?, all)?;
    for k in ["source_derived_rls_count", "live_rls_enabled_table_count"] {
        ensure_eq(k, n(k)?, rls)?;
    }
    for k in ["source_derived_force_rls_count", "live_force_rls_table_count"] {
        ensure_eq(k, n(k)?, force)?;
    }
    for k in ["source_derived_non_force_count", "live_non_force_all_awcms_table_count"] {
        ensure_eq(k, n(k)?, all - force)?;
    }
    ensure_eq("policy_count", n("policy_count")?, pol)?;
    // Hand-maintained register must not drift from the derived numbers.
    let m = c.json("manifest.json")?;
    let st = m.get("verifiedStaticSummary").ok_or("manifest.json: no verifiedStaticSummary")?;
    let lv = m.get("verifiedLiveSummary").ok_or("manifest.json: no verifiedLiveSummary")?;
    let sv = |j: &Json, k: &str| j.get(k).and_then(Json::num).ok_or(format!("manifest.json: missing {k}"));
    for (k, want) in [
        ("registeredModules", inv.modules.len() as u64),
        ("migrations", ledger),
        ("allAwcmsTablesIncludingMigrationLedger", all),
        ("forceRlsTables", force),
        ("nonForceAwcmsTablesIncludingMigrationLedger", all - force),
        ("testFiles", inv.tests),
        ("routeFiles", inv.routes.values().sum()),
    ] {
        ensure_eq(&format!("manifest.json verifiedStaticSummary.{k}"), sv(st, k)?, want)?;
    }
    for (k, want) in [
        ("migrationLedgerEntries", ledger),
        ("allAwcmsTablesIncludingMigrationLedger", all),
        ("rlsEnabledTables", rls),
        ("forceRlsTables", force),
        ("policies", pol),
    ] {
        ensure_eq(&format!("manifest.json verifiedLiveSummary.{k}"), sv(lv, k)?, want)?;
    }
    Ok(())
}

fn check_route_test_counts(c: &Ctx) -> R {
    let inv = load_inventory(c)?;
    ensure_eq("route surfaces sum vs summary Route files", inv.routes.values().sum::<u64>(), inv.summary.get("Route files").copied().unwrap_or(0))?;
    ensure_eq("test directory sum vs summary Test files", inv.tests, inv.summary.get("Test files").copied().unwrap_or(0))?;
    let oa = c.read(&format!("{SRC}/awcms-public-api.openapi.yaml"))?;
    let api_paths = oa.lines().filter(|l| l.starts_with("  /api/v1/") && l.ends_with(':')).count() as u64;
    ensure_eq("OpenAPI /api/v1 path keys vs inventory /api/v1/** route files", api_paths, inv.routes.get("/api/v1/**").copied().unwrap_or(0))?;
    Ok(())
}

struct Consumer {
    path: String,
    class: String,
}

fn load_consumer(c: &Ctx) -> Result<(Vec<Consumer>, Vec<Consumer>), String> {
    let j = c.json(&format!("{S}/astro-consumer.json"))?;
    let list = |k: &str| -> Result<Vec<Consumer>, String> {
        j.get(k)
            .and_then(Json::arr)
            .ok_or(format!("astro-consumer.json: no {k}"))?
            .iter()
            .map(|e| {
                Ok(Consumer {
                    path: e.get("path").and_then(Json::str).ok_or("entry without path")?.to_owned(),
                    class: e.get("execution_class").and_then(Json::str).ok_or("entry without execution_class")?.to_owned(),
                })
            })
            .collect()
    };
    Ok((list("consumed")?, list("committed")?))
}

fn check_astro_consumer(c: &Ctx) -> R {
    let (consumed, committed) = load_consumer(c)?;
    let ts = c.read(&format!("{SRC}/awcms-api-consumer-contract.ts"))?;
    let ts_consumed = ts_record_keys(&ts, "export const CONSUMED_PATHS", "export const COMMITTED_PATHS")?;
    let ts_committed = ts_record_keys(&ts, "export const COMMITTED_PATHS", "export const CONSUMER_PATHS")?;
    ensure_eq("consumed paths: fixture vs contract source", sorted(consumed.iter().map(|e| e.path.clone()).collect()), sorted(ts_consumed))?;
    ensure_eq("committed paths: fixture vs contract source", sorted(committed.iter().map(|e| e.path.clone()).collect()), sorted(ts_committed))?;
    ensure_eq("consumed count", consumed.len(), 13)?;
    ensure_eq("committed count", committed.len(), 2)?;
    ensure(committed.iter().all(|e| e.class == "future-consumer-contract"), "committed path with non-future class")?;
    let mut classes: BTreeMap<&str, usize> = BTreeMap::new();
    for e in &consumed {
        *classes.entry(e.class.as_str()).or_default() += 1;
    }
    let want: BTreeMap<&str, usize> = [
        ("build-time-machine-authenticated-read", 7),
        ("browser-anonymous-cross-origin-read", 2),
        ("browser-anonymous-cross-origin-telemetry-write", 1),
        ("browser-anonymous-cross-origin-security-sensitive-write", 3),
    ]
    .into_iter()
    .collect();
    ensure_eq("execution-class distribution of consumed paths", classes, want)?;
    let vectors = c.read(VECTORS)?;
    ensure(
        vectors.contains("\"id\": \"NEWSLETTER-PUBLIC-001\"") && vectors.contains("\"id\": \"ASTRO-TENANT-001\""),
        "execution classes (security-sensitive write / machine credential) have no matching authorization vectors",
    )
}

fn check_openapi_security(c: &Ctx) -> R {
    let (consumed, committed) = load_consumer(c)?;
    let oa = c.read(&format!("{SRC}/awcms-public-api.openapi.yaml"))?;
    let block = |path: &str| -> Result<Vec<&str>, String> {
        let head = format!("  {path}:");
        let mut it = oa.lines().skip_while(|l| *l != head);
        ensure(it.next().is_some(), format!("{path} absent from frozen OpenAPI"))?;
        Ok(it.take_while(|l| !l.starts_with("  /") && !l.starts_with("components:")).collect())
    };
    for e in consumed.iter().chain(committed.iter()) {
        let b = block(&e.path)?;
        let ops = b.iter().filter(|l| ["get", "post", "put", "patch", "delete"].iter().any(|m| **l == format!("    {m}:"))).count();
        let open = b.iter().filter(|l| l.trim_end() == "      security: []").count();
        ensure(ops > 0, format!("{}: no operations", e.path))?;
        if e.class.starts_with("browser-anonymous") {
            ensure_eq(&format!("{} ({}) anonymous operations (security: [])", e.path, e.class), open, ops)?;
        } else {
            ensure_eq(&format!("{} ({}) must inherit global bearer security", e.path, e.class), open, 0)?;
        }
    }
    Ok(())
}

fn check_vectors(c: &Ctx) -> R {
    let j = c.json(VECTORS)?;
    ensure_eq("vectors schemaVersion", j.get("schemaVersion").and_then(Json::num), Some(1))?;
    let vs = j.get("vectors").and_then(Json::arr).ok_or("vectors.json: no vectors")?;
    ensure_eq("vector count", vs.len(), 14)?;
    let mut ids = BTreeSet::new();
    for v in vs {
        let id = v.get("id").and_then(Json::str).ok_or("vector without id")?;
        ensure(ids.insert(id.to_owned()), format!("duplicate vector {id}"))?;
        for f in ["source", "class", "input", "expected"] {
            ensure(v.get(f).and_then(Json::str).is_some_and(|s| !s.trim().is_empty()), format!("{id}: empty {f}"))?;
        }
        ensure(
            v.get("source").and_then(Json::str).is_some_and(|s| {
                s.starts_with("ahliweb/awcms") || s.starts_with("controlled PostgreSQL introspection")
            }),
            format!("{id}: source is neither an AWCMS repository nor controlled DB introspection"),
        )?;
    }
    // The two DB vectors state numbers; bind them to the live fixtures.
    let inv = load_inventory(c)?;
    let force = inv.tables.iter().filter(|t| t.3).count();
    let expected = |id: &str, field: &str| {
        vs.iter()
            .find(|v| v.get("id").and_then(Json::str) == Some(id))
            .and_then(|v| v.get(field).and_then(Json::str))
            .unwrap_or("")
            .to_owned()
    };
    ensure(
        expected("DB-RLS-001", "expected").contains(&format!("{force} tables have RLS enabled and FORCE")),
        "DB-RLS-001 expected count differs from the FORCE-RLS count in the live fixtures",
    )?;
    ensure(
        expected("DB-RLS-001", "input").contains(&format!("{} migrations", inv.migrations.len())),
        "DB-RLS-001 input migration count differs from the migration inventory",
    )?;
    ensure_eq("vectors structuredSchemaVersion", j.get("structuredSchemaVersion").and_then(Json::num), Some(1))?;
    structured_vectors(c, vs)
}

fn check_manifest_digests(c: &Ctx) -> R {
    fn listed(text: &str) -> BTreeMap<String, String> {
        text.lines()
            .filter_map(|l| l.split_once("  "))
            .map(|(h, p)| (p.trim_start_matches("./").to_owned(), h.to_owned()))
            .collect()
    }
    fn files(dir: &Path, base: &Path, out: &mut BTreeSet<String>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    files(&p, base, out);
                } else if let Ok(r) = p.strip_prefix(base) {
                    out.insert(r.to_string_lossy().into_owned());
                }
            }
        }
    }
    for (dir, exclude) in [(S, "manifest.sha256"), (L, "manifest.sha256")] {
        let sha = listed(&c.read(&format!("{dir}/manifest.sha256"))?);
        let mut on_disk = BTreeSet::new();
        files(&c.root.join(dir), &c.root.join(dir), &mut on_disk);
        on_disk.remove(exclude);
        ensure_eq(&format!("{dir}: files on disk vs manifest.sha256 entries"), on_disk, sha.keys().cloned().collect())?;
        for (p, h) in &sha {
            ensure(is_sha256_hex(h), format!("{dir}/{p}: bad digest"))?;
        }
        if dir == S {
            let m = c.json(&format!("{S}/manifest.json"))?;
            let entries = m.get("files").and_then(Json::arr).ok_or("static/manifest.json: no files")?;
            let mut seen = BTreeSet::new();
            for e in entries {
                let p = e.get("path").and_then(Json::str).ok_or("manifest entry without path")?;
                let h = e.get("sha256").and_then(Json::str).ok_or("manifest entry without sha256")?;
                ensure_eq(&format!("static/manifest.json digest for {p}"), sha.get(p).map(String::as_str), Some(h))?;
                seen.insert(p.to_owned());
            }
            let mut want: BTreeSet<String> = sha.keys().cloned().collect();
            want.remove("manifest.json");
            ensure_eq("static/manifest.json file list vs manifest.sha256", seen, want)?;
        }
    }
    Ok(())
}

fn check_source_headers(c: &Ctx) -> R {
    let oa = c.read(&format!("{SRC}/awcms-public-api.openapi.yaml"))?;
    ensure(oa.lines().any(|l| l == "openapi: 3.1.0"), "frozen OpenAPI is not 3.1.0")?;
    let aa = c.read(&format!("{SRC}/awcms-domain-events.asyncapi.yaml"))?;
    ensure(aa.lines().next() == Some("asyncapi: 3.0.0"), "frozen AsyncAPI is not 3.0.0")?;
    ensure(c.read(&format!("{SRC}/awcms-ci.yml"))?.contains("run: bun run check"), "frozen AWCMS CI no longer shows the full `bun run check` gate")?;
    let t = c.read(&format!("{SRC}/awcms-astro-contract.test.mjs"))?;
    ensure(t.contains("tenant") && t.matches("test(").count() >= 5, "frozen consumer contract test lost its tenant-binding tests")?;
    let pv = |f: &str, name: &str, version: &str| -> R {
        let j = c.json(&format!("{SRC}/{f}"))?;
        ensure_eq(&format!("{f} name"), j.get("name").and_then(Json::str), Some(name))?;
        ensure_eq(&format!("{f} version"), j.get("version").and_then(Json::str), Some(version))
    };
    pv("awcms-package.json", "awcms", "10.1.0")?;
    pv("awcms-astro-package.json", "awcms-astro", "0.4.0")
}

// ------------------------------------------------------------ usage map ----

fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) -> Result<(), String> {
    for e in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = e.map_err(|e| e.to_string())?.path();
        if p.is_dir() {
            walk(&p, base, out)?;
        } else {
            out.push(p.strip_prefix(base).map_err(|e| e.to_string())?.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn check_usage_map(c: &Ctx) -> R {
    let j = c.json("fixture-usage.json")?;
    ensure_eq("fixture-usage schemaVersion", j.get("schemaVersion").and_then(Json::num), Some(1))?;
    let mut consumers: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    for e in j.get("consumers").and_then(Json::arr).ok_or("fixture-usage: no consumers")? {
        let f = |k: &str| e.get(k).and_then(Json::str).map(str::to_owned).ok_or(format!("consumer missing {k}"));
        let id = f("id")?;
        let strength = f("strength")?;
        ensure(["content", "structural", "integrity"].contains(&strength.as_str()), format!("{id}: bad strength {strength}"))?;
        ensure(consumers.insert(id.clone(), (f("file")?, f("marker")?, strength)).is_none(), format!("duplicate consumer {id}"))?;
    }
    for (id, (file, marker, _)) in &consumers {
        let text = c.repo_read(file).map_err(|e| format!("consumer {id}: file missing ({e})"))?;
        ensure(text.contains(marker.as_str()), format!("consumer {id}: marker '{marker}' not found in {file}"))?;
        if file == THIS_TOOL {
            let name = marker
                .strip_prefix("c.check(\"")
                .and_then(|m| m.strip_suffix("\","))
                .ok_or(format!("consumer {id}: tool marker must be c.check(\"<name>\","))?;
            ensure(c.ran.contains(name), format!("consumer {id}: check '{name}' did not run"))?;
        }
    }
    let mut on_disk = Vec::new();
    walk(&c.root.join("frozen"), &c.root, &mut on_disk)?;
    ensure(c.root.join(VECTORS).is_file(), "authorization vectors missing")?;
    on_disk.push(VECTORS.to_owned());
    let mut mapped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut used: BTreeSet<String> = BTreeSet::new();
    for a in j.get("artifacts").and_then(Json::arr).ok_or("fixture-usage: no artifacts")? {
        let path = a.get("path").and_then(Json::str).ok_or("artifact without path")?.to_owned();
        let ids: Vec<String> = a.get("consumers").and_then(Json::arr).ok_or(format!("{path}: no consumers list"))?
            .iter().filter_map(|x| x.str().map(str::to_owned)).collect();
        ensure(!ids.is_empty(), format!("orphan fixture: {path} lists no consumer"))?;
        for id in &ids {
            ensure(consumers.contains_key(id), format!("{path}: unknown consumer '{id}'"))?;
            used.insert(id.clone());
        }
        ensure(mapped.insert(path.clone(), ids).is_none(), format!("{path}: mapped twice"))?;
    }
    // A path ending in `/` maps a whole tree (frozen test copies are data-dependent).
    let covered = |file: &str| mapped.contains_key(file) || mapped.keys().any(|k| k.ends_with('/') && file.starts_with(k.as_str()));
    for p in &on_disk {
        ensure(covered(p), format!("orphan fixture: {p} is not in fixture-usage.json"))?;
    }
    for p in mapped.keys() {
        let present = if p.ends_with('/') { on_disk.iter().any(|f| f.starts_with(p.as_str())) } else { on_disk.contains(p) };
        ensure(present || c.pending.contains(p), format!("fixture-usage.json maps nonexistent file {p}"))?;
    }
    for p in &c.pending {
        ensure(mapped.contains_key(p), format!("pendingArtifacts entry {p} is not mapped in artifacts[]"))?;
    }
    for id in consumers.keys() {
        ensure(used.contains(id), format!("consumer '{id}' is declared but consumes no artifact"))?;
    }
    let mut weak = Vec::new();
    for (p, ids) in &mapped {
        if ids.iter().all(|i| consumers[i].2 == "integrity") {
            weak.push(p.clone());
        }
    }
    println!("info  usage-map: {} artifacts, {} consumers; integrity-only artifacts: {}", mapped.len(), consumers.len(), weak.len());
    for p in weak {
        println!("info    integrity-only: {p}");
    }
    Ok(())
}

// ------------------------------------------------- structured artifacts ----
//
// gates.json, routes.json, behaviour-fixtures.json and the frozen test copies
// are produced by tools/vg01_inventory.rs and arrive through the vg01-freeze
// workflow. Until they are committed, `fixture-usage.json` lists them in
// `pendingArtifacts`: an absent pending artifact is a note, a present one must
// validate and must be mapped. The freeze pull request empties that list.

const GATES: &str = "frozen/static/gates.json";
const ROUTES: &str = "frozen/static/routes.json";
const BEHAVIOUR: &str = "frozen/static/behaviour-fixtures.json";
const TEST_TREE: &str = "frozen/static/source/tests/awcms/";
const OPENAPI: &str = "frozen/static/source/awcms-public-api.openapi.yaml";

impl Ctx {
    fn note(&self, text: String) {
        let mut notes = self.notes.borrow_mut();
        if !notes.contains(&text) {
            notes.push(text);
        }
    }
}

/// `Ok(true)` when the artifact exists and must be validated, `Ok(false)` when
/// it is absent and listed as pending, an error when it is absent unlisted.
fn artifact_present(c: &Ctx, rel: &str) -> Result<bool, String> {
    let path = c.root.join(rel.trim_end_matches('/'));
    let exists = if rel.ends_with('/') {
        fs::read_dir(&path).map(|mut d| d.next().is_some()).unwrap_or(false)
    } else {
        path.is_file()
    };
    let pending = c.pending.contains(rel);
    match (exists, pending) {
        (true, true) => {
            c.note(format!("{rel} is present but still listed in pendingArtifacts (validated; the freeze PR empties the list)"));
            Ok(true)
        }
        (true, false) => Ok(true),
        (false, true) => {
            c.note(format!("{rel} is pending (not yet frozen); its validator was skipped"));
            Ok(false)
        }
        (false, false) => Err(format!("{rel} is absent and not listed in fixture-usage.json pendingArtifacts")),
    }
}

fn field<'a>(j: &'a Json, key: &str, what: &str) -> Result<&'a Json, String> {
    j.get(key).ok_or(format!("{what}: missing {key}"))
}

fn str_field<'a>(j: &'a Json, key: &str, what: &str) -> Result<&'a str, String> {
    field(j, key, what)?.str().ok_or(format!("{what}: {key} is not a string"))
}

fn arr_field<'a>(j: &'a Json, key: &str, what: &str) -> Result<&'a [Json], String> {
    field(j, key, what)?.arr().ok_or(format!("{what}: {key} is not an array"))
}

fn num_field(j: &Json, key: &str, what: &str) -> Result<u64, String> {
    field(j, key, what)?.num().ok_or(format!("{what}: {key} is not a non-negative integer"))
}

fn nonempty_strs(j: &Json, key: &str, what: &str) -> Result<Vec<String>, String> {
    let items = arr_field(j, key, what)?;
    ensure(!items.is_empty(), format!("{what}: {key} is empty"))?;
    items
        .iter()
        .map(|x| match x.str() {
            Some(s) if !s.trim().is_empty() => Ok(s.to_owned()),
            _ => Err(format!("{what}: {key} holds an empty or non-string entry")),
        })
        .collect()
}

fn same_sha(j: &Json, key: &str, want: &str, what: &str) -> R {
    ensure_eq(&format!("{what} {key}"), j.get(key).and_then(Json::str), Some(want))
}

/// `(METHOD, path with [param])` -> whether the operation declares `security: []`.
fn openapi_ops(text: &str) -> BTreeMap<(String, String), bool> {
    let mut out = BTreeMap::new();
    let mut in_paths = false;
    let mut path = String::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        if line == "paths:" {
            in_paths = true;
            continue;
        }
        if !in_paths {
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        if line.starts_with("  /") && !line.starts_with("   ") && line.ends_with(':') {
            path = line.trim().trim_end_matches(':').replace('{', "[").replace('}', "]");
            current = None;
        } else if let Some(m) = line.strip_prefix("    ").and_then(|l| l.strip_suffix(':')) {
            if !m.starts_with(' ') && ["get", "post", "put", "patch", "delete", "options", "head"].contains(&m) {
                let key = (m.to_ascii_uppercase(), path.clone());
                out.insert(key.clone(), false);
                current = Some(key);
            }
        } else if line == "      security: []"
            && let Some(key) = &current
        {
            out.insert(key.clone(), true);
        }
    }
    out
}

fn gate_like(name: &str) -> bool {
    let segments: Vec<&str> = name.split(':').collect();
    segments.iter().any(|s| matches!(*s, "check" | "verify"))
        || segments.last().is_some_and(|s| matches!(*s, "lint" | "typecheck" | "test" | "e2e" | "coverage" | "build"))
}

fn bun_run_names(text: &str) -> Vec<String> {
    text.match_indices("bun run ")
        .filter_map(|(at, marker)| {
            let name: String = text[at + marker.len()..]
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-'))
                .collect();
            let name = name.trim_end_matches(':').to_owned();
            if name.is_empty() { None } else { Some(name) }
        })
        .collect()
}

fn expand_chain(name: &str, scripts: &BTreeMap<String, String>, seen: &mut BTreeSet<String>) {
    if !seen.insert(name.to_owned()) {
        return;
    }
    if let Some(cmd) = scripts.get(name) {
        for inner in bun_run_names(cmd) {
            expand_chain(&inner, scripts, seen);
        }
    }
}

fn package_scripts(c: &Ctx) -> Result<BTreeMap<String, String>, String> {
    let pkg = c.json(&format!("{SRC}/awcms-package.json"))?;
    match pkg.get("scripts") {
        Some(Json::Obj(items)) => Ok(items.iter().filter_map(|(k, v)| v.str().map(|s| (k.clone(), s.to_owned()))).collect()),
        _ => Err("frozen AWCMS package.json has no scripts object".into()),
    }
}

fn check_gates(c: &Ctx) -> R {
    if !artifact_present(c, GATES)? {
        return Ok(());
    }
    let p = provenance(c)?;
    let j = c.json(GATES)?;
    same_sha(&j, "source_sha", &p.awcms, "gates.json")?;
    let scripts = package_scripts(c)?;
    let gates = arr_field(&j, "gates", "gates.json")?;
    let workflows = arr_field(&j, "workflows", "gates.json")?;

    // Workflow section vs the frozen raw CI file.
    let mut jobs_by_workflow: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut pr_blocking: BTreeMap<String, bool> = BTreeMap::new();
    for wf in workflows {
        let file = str_field(wf, "file", "gates.json workflow")?.to_owned();
        let jobs: BTreeSet<String> = arr_field(wf, "jobs", &file)?
            .iter()
            .filter_map(|job| job.get("id").and_then(Json::str).map(str::to_owned))
            .collect();
        pr_blocking.insert(file.clone(), matches!(wf.get("pr_blocking"), Some(Json::Bool(true))));
        jobs_by_workflow.insert(file, jobs);
    }
    let ci = c.read(&format!("{SRC}/awcms-ci.yml"))?;
    let mut raw_jobs = BTreeSet::new();
    let mut in_jobs = false;
    for line in ci.lines() {
        if line == "jobs:" {
            in_jobs = true;
        } else if in_jobs && !line.is_empty() && !line.starts_with(' ') && !line.starts_with('#') {
            in_jobs = false;
        } else if in_jobs && line.starts_with("  ") && !line.starts_with("   ") && line.trim_end().ends_with(':') && !line.trim_start().starts_with('#') {
            raw_jobs.insert(line.trim().trim_end_matches(':').to_owned());
        }
    }
    ensure_eq(
        "gates.json ci.yml jobs vs frozen awcms-ci.yml jobs",
        jobs_by_workflow.get(".github/workflows/ci.yml"),
        Some(&raw_jobs),
    )?;
    ensure(pr_blocking.get(".github/workflows/ci.yml") == Some(&true), "gates.json: ci.yml is not pr_blocking")?;

    let mut ids = BTreeSet::new();
    let mut tally: BTreeMap<&str, u64> = BTreeMap::new();
    let mut by_id: BTreeMap<&str, &Json> = BTreeMap::new();
    for g in gates {
        let id = str_field(g, "id", "gate")?;
        ensure(ids.insert(id.to_owned()), format!("duplicate gate {id}"))?;
        by_id.insert(id, g);
        let status = str_field(g, "status", id)?;
        ensure(["enforced", "advisory", "absent"].contains(&status), format!("{id}: bad status {status}"))?;
        *tally.entry(["enforced", "advisory", "absent"].iter().find(|s| **s == status).copied().unwrap_or("")).or_default() += 1;
        ensure(!str_field(g, "reason", id)?.trim().is_empty(), format!("{id}: empty reason"))?;
        ensure(!str_field(g, "kind", id)?.trim().is_empty(), format!("{id}: empty kind"))?;
        let source = nonempty_strs(g, "source", id)?;
        match str_field(g, "kind", id)? {
            "package-script" => {
                let name = id.strip_prefix("script:").ok_or(format!("{id}: package-script id must start with script:"))?;
                ensure_eq(&format!("{id} command vs frozen package.json"), g.get("command").and_then(Json::str), scripts.get(name).map(String::as_str))?;
                let ci_jobs = nonempty_or_empty(g, "ci_jobs", id)?;
                ensure_eq(&format!("{id}: enforced iff a pull-request job runs it"), status == "enforced", !ci_jobs.is_empty())?;
                for job in &ci_jobs {
                    let (wf, job_id) = job.split_once(':').ok_or(format!("{id}: bad job label {job}"))?;
                    let file = format!(".github/workflows/{wf}");
                    ensure(jobs_by_workflow.get(&file).is_some_and(|s| s.contains(job_id)), format!("{id}: job {job} not in workflows[]"))?;
                    ensure(pr_blocking.get(&file) == Some(&true), format!("{id}: ci_jobs names non-pull-request workflow {wf}"))?;
                }
                if let Some(file) = g.get("script_file").and_then(Json::str) {
                    ensure(source.iter().any(|s| s == file), format!("{id}: script_file {file} not among source paths"))?;
                }
                ensure(source.iter().any(|s| s == "package.json"), format!("{id}: source must cite package.json"))?;
            }
            "workflow-job" => {
                ensure_eq(&format!("{id} status"), status, "enforced")?;
                ensure(source.iter().all(|s| pr_blocking.get(s) == Some(&true)), format!("{id}: workflow is not a pull-request workflow"))?;
            }
            kind if kind.starts_with("documented-") => {
                ensure_eq(&format!("{id} status"), status, "absent")?;
                ensure(
                    source.iter().all(|s| s.rsplit_once(':').is_some_and(|(path, line)| !path.is_empty() && line.parse::<u64>().is_ok())),
                    format!("{id}: absent entries must cite path:line"),
                )?;
                if let Some(name) = id.strip_prefix("doc-ref:") {
                    ensure(!scripts.contains_key(name), format!("{id}: named script exists in the frozen package.json"))?;
                }
            }
            "script-file" => ensure_eq(&format!("{id} status"), status, "advisory")?,
            other => return Err(format!("{id}: unknown kind {other}")),
        }
    }
    let summary = field(&j, "summary", "gates.json")?;
    ensure_eq("gates summary total", num_field(summary, "total", "summary")?, gates.len() as u64)?;
    for status in ["enforced", "advisory", "absent"] {
        ensure_eq(&format!("gates summary {status}"), num_field(summary, status, "summary")?, tally.get(status).copied().unwrap_or(0))?;
    }

    // The `check` chain of the frozen package.json is the source of truth.
    let check_cmd = scripts.get("check").ok_or("frozen package.json has no check script")?;
    ensure(ci.contains("run: bun run check"), "frozen AWCMS CI does not run the full check chain")?;
    let mut chain = BTreeSet::new();
    expand_chain("check", &scripts, &mut chain);
    chain.remove("check");
    ensure_eq("gates summary check_chain_length", num_field(summary, "check_chain_length", "summary")?, chain.len() as u64)?;
    ensure_eq(
        "gates summary package_script_gates",
        num_field(summary, "package_script_gates", "summary")?,
        gates.iter().filter(|g| g.get("kind").and_then(Json::str) == Some("package-script")).count() as u64,
    )?;
    for item in check_cmd.split(" && ") {
        let Some(name) = item.trim().strip_prefix("bun run ") else { continue };
        if gate_like(name) {
            let g = by_id.get(format!("script:{name}").as_str()).ok_or(format!("check-chain gate script:{name} is missing from gates.json"))?;
            ensure_eq(&format!("script:{name} status"), g.get("status").and_then(Json::str), Some("enforced"))?;
            ensure_eq(&format!("script:{name} in_check_chain"), matches!(g.get("in_check_chain"), Some(Json::Bool(true))), true)?;
            ensure(
                nonempty_or_empty(g, "ci_jobs", name)?.iter().any(|job| job == "ci.yml:quality"),
                format!("script:{name} is in the check chain but not attributed to ci.yml:quality"),
            )?;
        }
    }
    ensure(tally.get("enforced").copied().unwrap_or(0) > 0 && tally.get("absent").copied().unwrap_or(0) > 0, "gate inventory must show both enforced and absent classes")
}

fn nonempty_or_empty(j: &Json, key: &str, what: &str) -> Result<Vec<String>, String> {
    arr_field(j, key, what)?
        .iter()
        .map(|x| x.str().map(str::to_owned).ok_or(format!("{what}: {key} holds a non-string")))
        .collect()
}

fn check_routes(c: &Ctx) -> R {
    if !artifact_present(c, ROUTES)? {
        return Ok(());
    }
    let p = provenance(c)?;
    let j = c.json(ROUTES)?;
    same_sha(&j, "source_sha", &p.awcms, "routes.json")?;
    let summary = field(&j, "summary", "routes.json")?;
    let routes = arr_field(&j, "routes", "routes.json")?;
    let inv = load_inventory(c)?;
    // Counts must agree with the frozen repo inventory.
    let inv_total = inv.summary.get("Route files").copied().unwrap_or(0);
    let inv_api = inv.routes.get("/api/v1/**").copied().unwrap_or(0);
    let inv_admin = inv.routes.get("/admin/**").copied().unwrap_or(0);
    ensure_eq("routes.json route_files vs inventory", num_field(summary, "route_files", "summary")?, inv_total)?;
    ensure_eq("routes.json api_route_files vs inventory", num_field(summary, "api_route_files", "summary")?, inv_api)?;
    ensure_eq("routes.json admin_route_files vs inventory", num_field(summary, "admin_route_files", "summary")?, inv_admin)?;
    ensure_eq("routes.json public_route_files vs inventory", num_field(summary, "public_route_files", "summary")?, inv_total - inv_api - inv_admin)?;
    ensure_eq("routes.json handlers", num_field(summary, "handlers", "summary")?, routes.len() as u64)?;

    let mut owners: BTreeSet<String> = inv.modules.iter().cloned().collect();
    owners.insert("platform".to_owned());
    owners.insert("core".to_owned());
    let oa = openapi_ops(&c.read(OPENAPI)?);
    let mut files: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    let mut class_tally: BTreeMap<String, u64> = BTreeMap::new();
    let mut owner_tally: BTreeMap<String, u64> = BTreeMap::new();
    let mut agreement: BTreeMap<String, u64> = BTreeMap::new();
    let mut seen_ops = BTreeSet::new();
    let mut seen_handlers = BTreeSet::new();
    for r in routes {
        let file = str_field(r, "file", "route")?;
        let route = str_field(r, "route", file)?;
        let method = str_field(r, "method", file)?;
        let what = format!("{method} {route}");
        ensure(seen_handlers.insert((file.to_owned(), method.to_owned())), format!("{what}: duplicate handler for {file}"))?;
        ensure(file.starts_with("src/pages/"), format!("{what}: file outside src/pages"))?;
        ensure(["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD", "ALL"].contains(&method), format!("{what}: bad method"))?;
        let surface = str_field(r, "surface", &what)?;
        ensure(["api", "admin", "public"].contains(&surface), format!("{what}: bad surface"))?;
        ensure_eq(&format!("{what} surface vs path"), surface, if route.starts_with("/api/") { "api" } else if route == "/admin" || route.starts_with("/admin/") { "admin" } else { "public" })?;
        files.entry((surface.to_owned(), file.to_owned())).or_default().insert(method.to_owned());
        let owner = str_field(r, "owner", &what)?;
        ensure(owners.contains(owner), format!("{what}: owner {owner} is neither a registered module nor platform/core"))?;
        ensure(!str_field(r, "owner_basis", &what)?.is_empty(), format!("{what}: empty owner_basis"))?;
        let class = str_field(r, "auth_class", &what)?;
        ensure(["public", "session", "machine-credential", "admin", "indeterminate"].contains(&class), format!("{what}: bad auth_class {class}"))?;
        ensure_eq(&format!("{what}: admin class iff admin surface"), class == "admin", surface == "admin")?;
        nonempty_strs(r, "auth_markers", &what)?;
        let machine = str_field(r, "machine_credential", &what)?;
        ensure(["accepted-by-guard", "refused-in-handler", "unspecified", "not-applicable"].contains(&machine), format!("{what}: bad machine_credential"))?;
        let verdict = str_field(r, "openapi_agreement", &what)?;
        ensure(["agrees", "differs", "indeterminate", "not-in-openapi", "not-applicable"].contains(&verdict), format!("{what}: bad openapi_agreement"))?;
        let security = str_field(r, "openapi_security", &what)?;
        if surface == "api" {
            match oa.get(&(method.to_owned(), route.to_owned())) {
                Some(none) => {
                    seen_ops.insert((method.to_owned(), route.to_owned()));
                    ensure_eq(&format!("{what}: openapi_security vs frozen OpenAPI"), security == "none", *none)?;
                    ensure(verdict != "not-in-openapi", format!("{what}: documented in OpenAPI but marked not-in-openapi"))?;
                    let agrees = match class {
                        "public" => *none,
                        "session" | "machine-credential" => !*none,
                        _ => false,
                    };
                    let expected = if class == "indeterminate" { "indeterminate" } else if agrees { "agrees" } else { "differs" };
                    ensure_eq(&format!("{what}: openapi_agreement recomputed from the frozen OpenAPI"), verdict, expected)?;
                }
                None => ensure_eq(&format!("{what}: not documented in the frozen OpenAPI"), verdict, "not-in-openapi")?,
            }
            if verdict == "differs" {
                ensure(r.get("note").and_then(Json::str).is_some_and(|n| !n.is_empty()), format!("{what}: differs from OpenAPI without an explanation (note)"))?;
            }
        } else {
            ensure_eq(&format!("{what}: non-API route"), verdict, "not-applicable")?;
        }
        *class_tally.entry(class.to_owned()).or_default() += 1;
        *owner_tally.entry(owner.to_owned()).or_default() += 1;
        *agreement.entry(verdict.to_owned()).or_default() += 1;
    }
    ensure_eq("routes.json distinct route files", files.len() as u64, inv_total)?;
    let ops: BTreeSet<(String, String)> = oa.keys().cloned().collect();
    ensure_eq("every frozen OpenAPI operation has a route handler", seen_ops, ops.clone())?;
    for module in &inv.modules {
        ensure(owner_tally.contains_key(module), format!("registered module {module} owns no route"))?;
    }
    let tally_of = |key: &str| -> Result<BTreeMap<String, u64>, String> {
        match field(summary, key, "summary")? {
            Json::Obj(items) => items.iter().map(|(k, v)| v.num().map(|n| (k.clone(), n)).ok_or(format!("summary.{key}.{k}"))).collect(),
            _ => Err(format!("summary.{key} is not an object")),
        }
    };
    ensure_eq("routes summary auth_class", tally_of("auth_class")?, class_tally)?;
    ensure_eq("routes summary owner", tally_of("owner")?, owner_tally)?;
    ensure_eq("routes summary openapi_agreement", tally_of("openapi_agreement")?, agreement)?;
    ensure_eq("routes summary openapi_operations", num_field(summary, "openapi_operations", "summary")?, ops.len() as u64)
}

fn check_behaviour(c: &Ctx) -> R {
    if !artifact_present(c, BEHAVIOUR)? {
        return Ok(());
    }
    let p = provenance(c)?;
    let j = c.json(BEHAVIOUR)?;
    same_sha(&j, "source_sha", &p.awcms, "behaviour-fixtures.json")?;
    same_sha(&j, "astro_source_sha", &p.astro, "behaviour-fixtures.json")?;
    let oa_text = c.read(OPENAPI)?;
    let oa = openapi_ops(&oa_text);

    // Classes: every one is stated, with an honest status.
    let mut classes = BTreeMap::new();
    for cl in arr_field(&j, "classes", "behaviour-fixtures.json")? {
        let name = str_field(cl, "class", "class")?;
        let status = str_field(cl, "status", name)?;
        ensure(["available", "available-by-reference", "not-available"].contains(&status), format!("{name}: bad status"))?;
        ensure(!str_field(cl, "what", name)?.is_empty(), format!("{name}: empty what"))?;
        classes.insert(name.to_owned(), (status.to_owned(), num_field(cl, "count", name)?));
    }
    for required in [
        "openapi-operation-contracts",
        "test-asserted-exchanges",
        "astro-consumer-mocked-fetch",
        "openapi-request-response-body-examples",
        "recorded-live-responses",
    ] {
        ensure(classes.contains_key(required), format!("behaviour class {required} is not stated"))?;
    }
    let example_blocks = oa_text.lines().filter(|l| l.trim_start().starts_with("examples:")).count() as u64;
    ensure_eq("body-examples class vs frozen OpenAPI `examples:` blocks", classes["openapi-request-response-body-examples"].1, example_blocks)?;
    ensure_eq(
        "body-examples class status",
        classes["openapi-request-response-body-examples"].0.as_str(),
        if example_blocks == 0 { "not-available" } else { "available" },
    )?;
    let astro_test = c.read(&format!("{SRC}/awcms-astro-contract.test.mjs"))?;
    ensure_eq("astro mocked-fetch count vs frozen consumer test", classes["astro-consumer-mocked-fetch"].1, astro_test.matches("globalThis.fetch = ").count() as u64)?;

    // OpenAPI operation contracts.
    let ops = arr_field(&j, "openapi_operations", "behaviour-fixtures.json")?;
    ensure_eq("openapi-operation-contracts count", classes["openapi-operation-contracts"].1, ops.len() as u64)?;
    let mut listed = BTreeSet::new();
    for o in ops {
        let method = str_field(o, "method", "operation")?;
        let path = str_field(o, "path", "operation")?;
        let what = format!("{method} {path}");
        listed.insert((method.to_owned(), path.to_owned()));
        let none = oa.get(&(method.to_owned(), path.to_owned())).ok_or(format!("{what}: not in the frozen OpenAPI"))?;
        ensure_eq(&format!("{what}: security vs frozen OpenAPI"), str_field(o, "security", &what)? == "none", *none)?;
        let responses = arr_field(o, "responses", &what)?;
        for r in responses {
            let text = r.str().ok_or(format!("{what}: response entry is not a string"))?;
            let code = text.split(' ').next().unwrap_or("");
            ensure(code == "default" || code.parse::<u16>().is_ok_and(|n| (100..=599).contains(&n)), format!("{what}: bad response status '{code}'"))?;
        }
        ensure(!responses.is_empty(), format!("{what}: no documented responses"))?;
    }
    ensure_eq("operation contracts vs frozen OpenAPI operations", listed, oa.keys().cloned().collect())?;

    // Frozen test tree: exact set, and every cited exchange resolves in it.
    let mut on_disk = Vec::new();
    if c.root.join(TEST_TREE.trim_end_matches('/')).is_dir() {
        walk(&c.root.join(TEST_TREE.trim_end_matches('/')), &c.root.join(TEST_TREE.trim_end_matches('/')), &mut on_disk)?;
    }
    on_disk.sort();
    let frozen_listed: Vec<String> = nonempty_strs(&j, "frozen_test_sources", "behaviour-fixtures.json")?;
    ensure_eq("frozen_test_sources vs files under the frozen test tree", sorted(frozen_listed.clone()), on_disk)?;
    let exchanges = arr_field(&j, "test_asserted_exchanges", "behaviour-fixtures.json")?;
    ensure_eq("test-asserted-exchanges count", classes["test-asserted-exchanges"].1, exchanges.len() as u64)?;
    ensure(!exchanges.is_empty() || classes["test-asserted-exchanges"].0 == "not-available", "no exchanges but the class is not marked not-available")?;
    let mut cited_files = BTreeSet::new();
    for e in exchanges {
        let test = str_field(e, "test", "exchange")?;
        let what = format!("exchange '{test}'");
        let method = str_field(e, "method", &what)?;
        ensure(["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"].contains(&method), format!("{what}: bad method"))?;
        ensure(str_field(e, "path", &what)?.starts_with('/'), format!("{what}: path must start with /"))?;
        ensure(["test-title", "describe-title", "request-literal"].contains(&str_field(e, "operation_source", &what)?), format!("{what}: bad operation_source"))?;
        let statuses = arr_field(e, "expected_status", &what)?;
        ensure(!statuses.is_empty(), format!("{what}: no expected status"))?;
        let source = str_field(e, "source", &what)?;
        let rest = source.strip_prefix("ahliweb/awcms:").ok_or(format!("{what}: source must start with ahliweb/awcms:"))?;
        let (rel, line) = rest.rsplit_once(':').ok_or(format!("{what}: source lacks :line"))?;
        let line: usize = line.parse().map_err(|_| format!("{what}: bad line number"))?;
        let copy = str_field(e, "frozen_copy", &what)?;
        ensure_eq(&format!("{what}: frozen_copy path"), copy, format!("source/tests/awcms/{rel}").as_str())?;
        cited_files.insert(rel.to_owned());
        let text = c.read(&format!("frozen/static/{copy}"))?;
        let lines: Vec<&str> = text.lines().collect();
        let at = |n: usize| lines.get(n).copied().unwrap_or("");
        ensure(
            at(line - 1).contains(test) || at(line).contains(test),
            format!("{what}: title not found at {rel}:{line} in the frozen copy"),
        )?;
        for s in statuses {
            let code = s.num().ok_or(format!("{what}: non-integer status"))?;
            ensure(
                text.contains(&format!(".status).toBe({code})")) || text.contains(&format!(".status).toEqual({code})")),
                format!("{what}: status {code} is not asserted in {rel}"),
            )?;
        }
    }
    let vectors = c.json(VECTORS)?;
    let vector_tests = vector_awcms_tests(arr_field(&vectors, "vectors", "vectors.json")?);
    for rel in &frozen_listed {
        ensure(
            cited_files.contains(rel) || vector_tests.contains(rel),
            format!("frozen test file {rel} is cited by no exchange or vector"),
        )?;
    }
    for rel in &vector_tests {
        ensure(frozen_listed.contains(rel), format!("vector-cited test file {rel} is not among frozen_test_sources"))?;
    }
    Ok(())
}

/// Repository-relative AWCMS test paths that authorization vectors cite.
fn vector_awcms_tests(vs: &[Json]) -> BTreeSet<String> {
    vs.iter()
        .filter_map(|v| v.get("structured")?.get("citation"))
        .filter(|cit| cit.get("repository").and_then(Json::str) == Some("ahliweb/awcms"))
        .filter_map(|cit| cit.get("path").and_then(Json::str).map(str::to_owned))
        .collect()
}

const DECISIONS: [&str; 5] = ["allow", "deny", "fail-closed", "neutral-response", "invariant"];
const ACTORS: [&str; 3] = ["machine-credential", "anonymous", "database-role"];
const TENANT_RELATIONS: [&str; 6] = [
    "credential-bound",
    "credential-bound-header-matching",
    "credential-bound-header-conflicting",
    "host-or-verified-origin-derived",
    "not-applicable",
    "schema-wide",
];
const TEST_KINDS: [&str; 3] = ["behavioural-unit", "static-source-assertion", "introspection"];

/// Structured fields of every authorization vector (applies immediately; the
/// citations into not-yet-frozen test copies are pending-gated).
fn structured_vectors(c: &Ctx, vs: &[Json]) -> R {
    let oa = openapi_ops(&c.read(OPENAPI)?);
    let oa_paths: BTreeSet<&str> = oa.keys().map(|(_, p)| p.as_str()).collect();
    for v in vs {
        let id = str_field(v, "id", "vector")?;
        let s = v.get("structured").ok_or(format!("{id}: missing structured block"))?;
        let what = format!("{id}.structured");
        ensure(ACTORS.contains(&str_field(s, "actor", &what)?), format!("{what}: bad actor"))?;
        ensure(!str_field(s, "credentialClass", &what)?.trim().is_empty(), format!("{what}: empty credentialClass"))?;
        ensure(TENANT_RELATIONS.contains(&str_field(s, "tenantRelation", &what)?), format!("{what}: bad tenantRelation"))?;
        ensure(DECISIONS.contains(&str_field(s, "expectedDecision", &what)?), format!("{what}: bad expectedDecision"))?;
        ensure(TEST_KINDS.contains(&str_field(s, "testKind", &what)?), format!("{what}: bad testKind"))?;
        let ops = arr_field(s, "operations", &what)?;
        ensure(!ops.is_empty(), format!("{what}: no operations"))?;
        let mut http = false;
        for op in ops {
            match str_field(op, "kind", &what)? {
                "http" => {
                    http = true;
                    let method = str_field(op, "method", &what)?;
                    let path = str_field(op, "path", &what)?;
                    ensure(["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"].contains(&method), format!("{what}: bad method {method}"))?;
                    ensure(oa_paths.contains(path), format!("{what}: {path} is not a path of the frozen OpenAPI"))?;
                    if method != "OPTIONS" {
                        ensure(oa.contains_key(&(method.to_owned(), path.to_owned())), format!("{what}: {method} {path} is not an operation of the frozen OpenAPI"))?;
                    }
                }
                "build-function" => {
                    for f in ["repository", "file", "name"] {
                        ensure(!str_field(op, f, &what)?.trim().is_empty(), format!("{what}: build-function {f} empty"))?;
                    }
                }
                "database-introspection" => {}
                other => return Err(format!("{what}: unknown operation kind {other}")),
            }
        }
        match s.get("expectedStatus") {
            Some(Json::Null) => {}
            Some(n) => {
                ensure(http, format!("{what}: expectedStatus set without an http operation"))?;
                ensure(n.num().is_some_and(|n| (100..=599).contains(&n)), format!("{what}: expectedStatus is not an HTTP status"))?;
            }
            None => return Err(format!("{what}: missing expectedStatus (use null when none)")),
        }
        let cit = field(s, "citation", &what)?;
        if let Some(files) = cit.get("frozenFiles") {
            ensure_eq(&format!("{what}: evidence kind"), str_field(cit, "evidence", &what)?, "live-introspection")?;
            ensure(!files.arr().unwrap_or(&[]).is_empty(), format!("{what}: no frozenFiles"))?;
            for f in files.arr().unwrap_or(&[]) {
                let rel = f.str().ok_or(format!("{what}: frozenFiles entry is not a string"))?;
                ensure(c.root.join(rel).is_file(), format!("{what}: frozen file {rel} does not exist"))?;
            }
            ensure_eq(&format!("{what}: testKind for live evidence"), str_field(s, "testKind", &what)?, "introspection")?;
        } else {
            let repo = str_field(cit, "repository", &what)?;
            ensure(["ahliweb/awcms", "ahliweb/awcms-astro"].contains(&repo), format!("{what}: bad citation repository"))?;
            let path = str_field(cit, "path", &what)?;
            let test = str_field(cit, "test", &what)?;
            ensure(!test.trim().is_empty(), format!("{what}: empty cited test name"))?;
            ensure_eq(&format!("{what}: prose source vs citation"), str_field(v, "source", id)?, format!("{repo}:{path}").as_str())?;
            let copy = str_field(cit, "frozenCopy", &what)?;
            ensure(copy.starts_with("frozen/static/source/"), format!("{what}: frozenCopy must live under frozen/static/source/"))?;
            if c.root.join(copy).is_file() {
                let text = c.read(copy)?;
                ensure(text.contains(test), format!("{what}: cited test '{test}' does not appear in {copy}"))?;
            } else if c.pending.contains(TEST_TREE) && copy.starts_with(TEST_TREE) {
                c.note(format!("citation target {copy} is pending (vector citations resolve after the freeze)"));
            } else {
                return Err(format!("{what}: frozen copy {copy} does not exist and is not pending"));
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("usage: vg01_check_corpus <contracts/legacy/awcms> [repo-root]");
        return ExitCode::from(2);
    }
    let root = PathBuf::from(&args[1]);
    let pending: BTreeSet<String> = fs::read_to_string(root.join("fixture-usage.json"))
        .ok()
        .and_then(|text| parse_json(&text).ok())
        .and_then(|j| j.get("pendingArtifacts").and_then(Json::arr).map(|a| a.iter().filter_map(|x| x.str().map(str::to_owned)).collect()))
        .unwrap_or_default();
    let mut c = Ctx {
        root,
        repo: PathBuf::from(args.get(2).map_or(".", String::as_str)),
        failures: Vec::new(),
        ran: BTreeSet::new(),
        notes: RefCell::new(Vec::new()),
        pending,
    };
    c.check("stale-source-sha", check_stale_sha);
    c.check("modules-registry", check_modules);
    c.check("migrations", check_migrations);
    c.check("tables-rls", check_tables_rls);
    c.check("static-vs-live", check_static_vs_live);
    c.check("live-roles", check_live_roles);
    c.check("live-summary", check_live_summary);
    c.check("route-test-counts", check_route_test_counts);
    c.check("astro-consumer", check_astro_consumer);
    c.check("openapi-security", check_openapi_security);
    c.check("vectors", check_vectors);
    c.check("manifest-digests", check_manifest_digests);
    c.check("source-headers", check_source_headers);
    c.check("gates", check_gates);
    c.check("routes", check_routes);
    c.check("behaviour-fixtures", check_behaviour);
    c.check("usage-map", check_usage_map);
    c.note("authorization vectors carry structured fields and cited tests; no parity harness executes them yet".into());
    for n in c.notes.borrow().iter() {
        println!("note  {n}");
    }
    if c.failures.is_empty() {
        println!("VG-01 corpus consumer: {} checks passed", c.ran.len());
        ExitCode::SUCCESS
    } else {
        eprintln!("VG-01 corpus consumer: {} of {} checks failed", c.failures.len(), c.ran.len());
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip_shapes() {
        let j = parse_json(r#"{"a":[1,"x\n",true,null],"b":{"c":"d"}}"#).unwrap();
        assert_eq!(j.get("b").and_then(|b| b.get("c")).and_then(Json::str), Some("d"));
        assert_eq!(j.get("a").and_then(Json::arr).map(<[Json]>::len), Some(4));
        assert!(parse_json("{\"a\":1} x").is_err());
    }

    #[test]
    fn csv_handles_quotes_and_newlines() {
        let r = parse_csv("a,b\n\"x,1\",\"y\"\"z\nw\"\n");
        assert_eq!(r[1], vec!["x,1".to_owned(), "y\"z\nw".to_owned()]);
    }

    #[test]
    fn openapi_ops_reads_methods_and_open_security() {
        let yaml = "paths:\n  /api/v1/x/{id}:\n    get:\n      security: []\n      responses: {}\n    post:\n      responses: {}\ncomponents:\n  a: b\n";
        let ops = openapi_ops(yaml);
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[&("GET".to_owned(), "/api/v1/x/[id]".to_owned())], true);
        assert_eq!(ops[&("POST".to_owned(), "/api/v1/x/[id]".to_owned())], false);
    }

    #[test]
    fn gate_names_and_chains() {
        assert!(gate_like("modules:dag:check") && gate_like("typecheck") && !gate_like("db:migrate"));
        let scripts: BTreeMap<String, String> = [("check", "bun run a:check && bun run build"), ("build", "bun run build:x"), ("a:check", "x"), ("build:x", "y")]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        let mut seen = BTreeSet::new();
        expand_chain("check", &scripts, &mut seen);
        assert_eq!(seen.len(), 4);
        assert_eq!(bun_run_names("bun run a:check && bun run i18n:"), vec!["a:check", "i18n"]);
    }

    #[test]
    fn pending_artifacts_gate_absence() {
        let root = env::temp_dir().join(format!("awbms-vg01-pending-{}", std::process::id()));
        fs::create_dir_all(root.join("frozen/static")).unwrap();
        fs::write(root.join("frozen/static/present.json"), "{}").unwrap();
        let mk = |pending: &[&str]| Ctx {
            root: root.clone(),
            repo: root.clone(),
            failures: Vec::new(),
            ran: BTreeSet::new(),
            notes: RefCell::new(Vec::new()),
            pending: pending.iter().map(|s| (*s).to_owned()).collect(),
        };
        let listed = mk(&["frozen/static/absent.json"]);
        assert_eq!(artifact_present(&listed, "frozen/static/absent.json"), Ok(false));
        assert_eq!(artifact_present(&listed, "frozen/static/present.json"), Ok(true));
        let unlisted = mk(&[]);
        assert!(artifact_present(&unlisted, "frozen/static/absent.json").is_err());
        assert_eq!(artifact_present(&mk(&["frozen/static/present.json"]), "frozen/static/present.json"), Ok(true));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sha_shape() {
        assert!(is_sha1_hex(&"a".repeat(40)));
        assert!(!is_sha1_hex(&"A".repeat(40)));
        assert!(is_sha256_hex(&"0".repeat(64)));
    }
}

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
    notes: Vec<String>,
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
    )
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
    for p in &on_disk {
        ensure(mapped.contains_key(p), format!("orphan fixture: {p} is not in fixture-usage.json"))?;
    }
    for p in mapped.keys() {
        ensure(on_disk.contains(p), format!("fixture-usage.json maps nonexistent file {p}"))?;
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

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("usage: vg01_check_corpus <contracts/legacy/awcms> [repo-root]");
        return ExitCode::from(2);
    }
    let mut c = Ctx {
        root: PathBuf::from(&args[1]),
        repo: PathBuf::from(args.get(2).map_or(".", String::as_str)),
        failures: Vec::new(),
        ran: BTreeSet::new(),
        notes: Vec::new(),
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
    c.check("usage-map", check_usage_map);
    c.notes.push("authorization vectors are prose (input/expected strings); no parity harness executes them yet".into());
    for n in &c.notes {
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
    fn sha_shape() {
        assert!(is_sha1_hex(&"a".repeat(40)));
        assert!(!is_sha1_hex(&"A".repeat(40)));
        assert!(is_sha256_hex(&"0".repeat(64)));
    }
}

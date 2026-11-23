use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

const AWCMS_SHA: &str = "11f2e95a47b1328a820f976d60f978c38a067903";
const ASTRO_SHA: &str = "7b753be619244541b817d5d8e7d3b72cfe88d4f9";
const SNAPSHOT_DATE: &str = "2026-08-29";
const EXPECTED_MODULES: usize = 24;
const EXPECTED_MIGRATIONS: usize = 148;
const EXPECTED_CONSUMED_PATHS: usize = 13;
const EXPECTED_COMMITTED_PATHS: usize = 2;
const EXPECTED_ROUTE_FILES: [(&str, usize); 3] = [("api", 308), ("admin", 50), ("public", 30)];
/// Frozen copies of cited AWCMS test files live under `source/tests/awcms/`,
/// keeping the repository-relative path (for example `tests/x.test.ts`).
const FROZEN_TEST_ROOT: &str = "source/tests/awcms";
/// Test files that authorization vectors cite; always frozen, whether or not
/// they also hold behaviour exchanges.
const VECTOR_TEST_SOURCES: [&str; 1] = ["tests/newsletter-subscription.test.ts"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err("usage: vg01_inventory <awcms-checkout> <awcms-astro-checkout> <output-dir>".into());
    }

    let awcms = Path::new(&args[1]);
    let astro = Path::new(&args[2]);
    let output = Path::new(&args[3]);

    verify_git_head(awcms, AWCMS_SHA)?;
    verify_git_head(astro, ASTRO_SHA)?;

    if output.exists() {
        fs::remove_dir_all(output)?;
    }
    fs::create_dir_all(output)?;

    let module_source = awcms.join("src/modules/index.ts");
    let module_source_text = fs::read_to_string(&module_source)?;
    let mut modules = parse_module_imports(&module_source_text);
    modules.sort();
    ensure_count("registered modules", modules.len(), EXPECTED_MODULES)?;
    write_string_array(&output.join("modules.json"), "modules", &modules)?;

    let migrations = collect_migrations(&awcms.join("sql"))?;
    ensure_count("migrations", migrations.len(), EXPECTED_MIGRATIONS)?;
    write_migrations(&output.join("migrations.json"), &migrations)?;

    let consumer_source = awcms.join("scripts/api-consumer-contract.ts");
    let consumer_text = fs::read_to_string(&consumer_source)?;
    let consumed = parse_record_paths(
        &consumer_text,
        "export const CONSUMED_PATHS",
        "export const COMMITTED_PATHS",
    )?;
    let committed = parse_record_paths(
        &consumer_text,
        "export const COMMITTED_PATHS",
        "export const CONSUMER_PATHS",
    )?;
    ensure_count(
        "AWCMS-Astro consumed paths",
        consumed.len(),
        EXPECTED_CONSUMED_PATHS,
    )?;
    ensure_count(
        "AWCMS-Astro committed paths",
        committed.len(),
        EXPECTED_COMMITTED_PATHS,
    )?;
    write_consumer_inventory(&output.join("astro-consumer.json"), &consumed, &committed)?;

    let frozen_sources = [
        (
            awcms.join("src/modules/index.ts"),
            "source/awcms-module-registry.ts",
        ),
        (
            awcms.join("docs/awcms/repo-inventory.md"),
            "source/awcms-repo-inventory.md",
        ),
        (
            awcms.join("openapi/awcms-public-api.openapi.yaml"),
            "source/awcms-public-api.openapi.yaml",
        ),
        (
            awcms.join("asyncapi/awcms-domain-events.asyncapi.yaml"),
            "source/awcms-domain-events.asyncapi.yaml",
        ),
        (
            awcms.join("scripts/api-consumer-contract.ts"),
            "source/awcms-api-consumer-contract.ts",
        ),
        (awcms.join("package.json"), "source/awcms-package.json"),
        (
            awcms.join(".github/workflows/ci.yml"),
            "source/awcms-ci.yml",
        ),
        (
            astro.join("tests/kontrak-awcms.test.mjs"),
            "source/awcms-astro-contract.test.mjs",
        ),
        (
            astro.join("package.json"),
            "source/awcms-astro-package.json",
        ),
    ];

    for (source, relative) in frozen_sources {
        copy_file(&source, &output.join(relative))?;
    }

    // Structured inventories derived from the pinned source (criteria 1, 3, 4
    // of issue #3). Everything below is parsed from files, never from prose.
    let gates = build_gates(awcms)?;
    write_json(&output.join("gates.json"), &gates.json)?;
    let routes = build_routes(awcms)?;
    write_json(&output.join("routes.json"), &routes.json)?;
    let behaviour = build_behaviour(awcms, astro)?;
    write_json(&output.join("behaviour-fixtures.json"), &behaviour.json)?;
    // Freeze the cited AWCMS test files so citations resolve inside the corpus.
    for relative in &behaviour.frozen_tests {
        copy_file(
            &awcms.join(relative),
            &output.join(FROZEN_TEST_ROOT).join(relative),
        )?;
    }

    // Implementation facts: rate limits and credential formats (item 7 of the
    // VG-01 evidence set), with the cited AWCMS files frozen beside them.
    let impl_files = load_impl_files(awcms)?;
    let rate_book = Book::new(&impl_files);
    let rates = build_rate_limits(&rate_book)?;
    let credential_book = Book::new(&impl_files);
    let credentials = build_credentials(&credential_book)?;
    let mut frozen_impl: BTreeSet<String> = rates.used.iter().cloned().collect();
    frozen_impl.extend(credentials.used.iter().cloned());
    write_json(&output.join("rate-limits.json"), &with_sources(rates.json, &rates.used))?;
    write_json(&output.join("credential-formats.json"), &with_sources(credentials.json, &credentials.used))?;
    for relative in &frozen_impl {
        copy_file(&awcms.join(relative), &output.join(FROZEN_IMPL_ROOT).join(relative))?;
    }

    write_manifest(output)?;

    println!(
        "VG-01 implementation inventories: {} limiter call lines, {} buckets in {} endpoint classes; {} credential-like columns, {} credential kinds, {} frozen implementation files",
        rates.sites,
        rates.buckets,
        rates.classes,
        credentials.columns,
        credentials.kinds,
        frozen_impl.len()
    );
    println!(
        "VG-01 static inventory generated: {EXPECTED_MODULES} modules, {EXPECTED_MIGRATIONS} migrations, {EXPECTED_CONSUMED_PATHS} consumed paths, {EXPECTED_COMMITTED_PATHS} committed paths"
    );
    println!(
        "VG-01 structured inventories: {} gates, {} route handlers in {} route files, {} OpenAPI operations, {} test-asserted exchanges, {} frozen test files",
        gates.count,
        routes.handlers,
        routes.files,
        behaviour.operations,
        behaviour.exchanges,
        behaviour.frozen_tests.len()
    );
    Ok(())
}

fn verify_git_head(repo: &Path, expected: &str) -> Result<(), Box<dyn std::error::Error>> {
    let result = Command::new("git")
        .args([
            "-C",
            repo.to_str().ok_or("non-UTF-8 checkout path")?,
            "rev-parse",
            "HEAD",
        ])
        .output()?;
    if !result.status.success() {
        return Err(format!("git rev-parse failed for {}", repo.display()).into());
    }
    let actual = String::from_utf8(result.stdout)?.trim().to_owned();
    if actual != expected {
        return Err(format!(
            "stale/wrong source checkout {}: expected {expected}, got {actual}",
            repo.display()
        )
        .into());
    }
    Ok(())
}

fn parse_module_imports(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with("import {")
                || !line.contains(" from \"./")
                || !line.ends_with("/module\";")
            {
                return None;
            }
            let start = line.find("from \"./")? + "from \"./".len();
            let rest = &line[start..];
            let end = rest.find("/module\"")?;
            Some(rest[..end].to_owned())
        })
        .collect()
}

fn collect_migrations(
    sql_dir: &Path,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("sql")))
        .collect();
    paths.sort();

    let mut migrations = Vec::with_capacity(paths.len());
    for path in paths {
        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or("non-UTF-8 migration path")?
            .to_owned();
        migrations.push((name, sha256_file(&path)?));
    }
    Ok(migrations)
}

fn parse_record_paths(
    source: &str,
    start_marker: &str,
    end_marker: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let start = source
        .find(start_marker)
        .ok_or_else(|| format!("missing marker {start_marker}"))?;
    let tail = &source[start..];
    let end = tail
        .find(end_marker)
        .ok_or_else(|| format!("missing marker {end_marker}"))?;
    let block = &tail[..end];
    let mut paths = Vec::new();
    for line in block.lines() {
        let line = line.trim_start();
        if !line.starts_with("\"/api/") {
            continue;
        }
        if let Some(end_quote) = line[1..].find('"') {
            paths.push(line[1..1 + end_quote].to_owned());
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn classify_consumer(path: &str) -> &'static str {
    match path {
        "/api/v1/site-search/query" | "/api/v1/site-search/suggest" => {
            "browser-anonymous-cross-origin-read"
        }
        "/api/v1/analytics/collect" => "browser-anonymous-cross-origin-telemetry-write",
        "/api/v1/newsletter/subscribe"
        | "/api/v1/newsletter/confirm"
        | "/api/v1/newsletter/unsubscribe" => {
            "browser-anonymous-cross-origin-security-sensitive-write"
        }
        _ => "build-time-machine-authenticated-read",
    }
}

fn ensure_count(
    label: &str,
    actual: usize,
    expected: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if actual != expected {
        return Err(format!("{label} count drift: expected {expected}, got {actual}").into());
    }
    Ok(())
}

fn copy_file(source: &Path, target: &Path) -> io::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, target)?;
    Ok(())
}

fn write_string_array(path: &Path, key: &str, values: &[String]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    writeln!(file, "{{")?;
    writeln!(file, "  \"source_sha\": \"{AWCMS_SHA}\",")?;
    writeln!(file, "  \"count\": {},", values.len())?;
    writeln!(file, "  \"{key}\": [")?;
    for (index, value) in values.iter().enumerate() {
        let comma = if index + 1 == values.len() { "" } else { "," };
        writeln!(file, "    \"{}\"{comma}", json_escape(value))?;
    }
    writeln!(file, "  ]")?;
    writeln!(file, "}}")
}

fn write_migrations(path: &Path, migrations: &[(String, String)]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    writeln!(file, "{{")?;
    writeln!(file, "  \"source_sha\": \"{AWCMS_SHA}\",")?;
    writeln!(file, "  \"count\": {},", migrations.len())?;
    writeln!(file, "  \"migrations\": [")?;
    for (index, (name, digest)) in migrations.iter().enumerate() {
        let comma = if index + 1 == migrations.len() { "" } else { "," };
        writeln!(
            file,
            "    {{\"path\": \"sql/{}\", \"sha256\": \"{}\"}}{comma}",
            json_escape(name),
            digest
        )?;
    }
    writeln!(file, "  ]")?;
    writeln!(file, "}}")
}

fn write_consumer_inventory(
    path: &Path,
    consumed: &[String],
    committed: &[String],
) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    writeln!(file, "{{")?;
    writeln!(file, "  \"awcms_source_sha\": \"{AWCMS_SHA}\",")?;
    writeln!(file, "  \"astro_source_sha\": \"{ASTRO_SHA}\",")?;
    writeln!(file, "  \"consumed\": [")?;
    for (index, value) in consumed.iter().enumerate() {
        let comma = if index + 1 == consumed.len() { "" } else { "," };
        writeln!(
            file,
            "    {{\"path\": \"{}\", \"execution_class\": \"{}\"}}{comma}",
            json_escape(value),
            classify_consumer(value)
        )?;
    }
    writeln!(file, "  ],")?;
    writeln!(file, "  \"committed\": [")?;
    for (index, value) in committed.iter().enumerate() {
        let comma = if index + 1 == committed.len() { "" } else { "," };
        writeln!(
            file,
            "    {{\"path\": \"{}\", \"execution_class\": \"future-consumer-contract\"}}{comma}",
            json_escape(value)
        )?;
    }
    writeln!(file, "  ]")?;
    writeln!(file, "}}")
}

fn write_manifest(output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_files(output, output, &mut files)?;
    files.retain(|(path, _)| path != "manifest.json");
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut file = fs::File::create(output.join("manifest.json"))?;
    writeln!(file, "{{")?;
    writeln!(file, "  \"gate\": \"VG-01\",")?;
    writeln!(
        file,
        "  \"status\": \"partial-static-source-evidence\"," 
    )?;
    writeln!(file, "  \"snapshot_date\": \"{SNAPSHOT_DATE}\",")?;
    writeln!(file, "  \"sources\": [")?;
    writeln!(
        file,
        "    {{\"repository\": \"ahliweb/awcms\", \"commit\": \"{AWCMS_SHA}\"}},"
    )?;
    writeln!(
        file,
        "    {{\"repository\": \"ahliweb/awcms-astro\", \"commit\": \"{ASTRO_SHA}\"}}"
    )?;
    writeln!(file, "  ],")?;
    writeln!(file, "  \"files\": [")?;
    for (index, (path, digest)) in files.iter().enumerate() {
        let comma = if index + 1 == files.len() { "" } else { "," };
        writeln!(
            file,
            "    {{\"path\": \"{}\", \"sha256\": \"{}\"}}{comma}",
            json_escape(path),
            digest
        )?;
    }
    writeln!(file, "  ],")?;
    writeln!(
        file,
        "  \"remaining_live_evidence\": [\"controlled PostgreSQL RLS introspection\", \"controlled PostgreSQL roles/grants introspection\", \"authorization vectors and representative parity fixtures\"]"
    )?;
    writeln!(file, "}}")?;
    Ok(())
}

fn collect_files(
    root: &Path,
    current: &Path,
    out: &mut Vec<(String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries: Vec<_> = fs::read_dir(current)?.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, sha256_file(&path)?));
        }
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let result = Command::new("sha256sum").arg(path).output()?;
    if !result.status.success() {
        return Err(format!("sha256sum failed for {}", path.display()).into());
    }
    let stdout = String::from_utf8(result.stdout)?;
    let digest = stdout
        .split_whitespace()
        .next()
        .ok_or("sha256sum produced no digest")?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("invalid SHA-256 output for {}", path.display()).into());
    }
    Ok(digest.to_ascii_lowercase())
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

// ===================================================================== JSON ==

/// Minimal JSON value with a deterministic pretty-printer. Objects keep the
/// insertion order they are built with; callers insert in a fixed order.
enum J {
    S(String),
    N(i64),
    B(bool),
    Null,
    A(Vec<J>),
    O(Vec<(String, J)>),
}

fn js(value: &str) -> J {
    J::S(value.to_owned())
}

fn jn(value: usize) -> J {
    J::N(i64::try_from(value).unwrap_or(i64::MAX))
}

fn jo(items: Vec<(&str, J)>) -> J {
    J::O(items.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}

fn jstrs<S: AsRef<str>>(values: &[S]) -> J {
    J::A(values.iter().map(|v| js(v.as_ref())).collect())
}

fn jopt(value: Option<&str>) -> J {
    value.map_or(J::Null, js)
}

fn is_scalar(j: &J) -> bool {
    matches!(j, J::S(_) | J::N(_) | J::B(_) | J::Null)
}

fn is_inline(j: &J) -> bool {
    match j {
        J::A(items) => items.iter().all(is_scalar),
        J::O(items) => items.iter().all(|(_, v)| is_scalar(v) || matches!(v, J::A(a) if a.iter().all(is_scalar))),
        _ => true,
    }
}

fn json_str(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn render_json(j: &J, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth + 1);
    let close = "  ".repeat(depth);
    match j {
        J::S(v) => out.push_str(&json_str(v)),
        J::N(v) => out.push_str(&v.to_string()),
        J::B(v) => out.push_str(if *v { "true" } else { "false" }),
        J::Null => out.push_str("null"),
        J::A(items) if items.is_empty() => out.push_str("[]"),
        J::O(items) if items.is_empty() => out.push_str("{}"),
        J::A(items) => {
            if is_inline(j) {
                out.push('[');
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    render_json(item, depth, out);
                }
                out.push(']');
            } else {
                out.push_str("[\n");
                for (index, item) in items.iter().enumerate() {
                    out.push_str(&pad);
                    render_json(item, depth + 1, out);
                    out.push_str(if index + 1 == items.len() { "\n" } else { ",\n" });
                }
                out.push_str(&close);
                out.push(']');
            }
        }
        J::O(items) => {
            if is_inline(j) {
                out.push('{');
                for (index, (key, value)) in items.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&json_str(key));
                    out.push_str(": ");
                    render_json(value, depth, out);
                }
                out.push('}');
            } else {
                out.push_str("{\n");
                for (index, (key, value)) in items.iter().enumerate() {
                    out.push_str(&pad);
                    out.push_str(&json_str(key));
                    out.push_str(": ");
                    render_json(value, depth + 1, out);
                    out.push_str(if index + 1 == items.len() { "\n" } else { ",\n" });
                }
                out.push_str(&close);
                out.push('}');
            }
        }
    }
}

fn write_json(path: &Path, value: &J) -> io::Result<()> {
    let mut text = String::new();
    render_json(value, 0, &mut text);
    text.push('\n');
    fs::write(path, text)
}

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn read_text(path: &Path) -> Res<String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()).into())
}

/// Regular files below `root`, as `/`-separated paths relative to it, sorted.
fn list_files(root: &Path) -> Res<Vec<String>> {
    fn walk(root: &Path, current: &Path, out: &mut Vec<String>) -> Res<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let path = entry.path();
            if kind.is_dir() {
                walk(root, &path, out)?;
            } else if kind.is_file() {
                out.push(path.strip_prefix(root)?.to_string_lossy().replace('\\', "/"));
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

/// Drops whole-line comments (`//`, `/*`, ` * `); keeps everything else.
fn strip_comment_lines(text: &str) -> String {
    text.lines()
        .filter(|l| {
            let t = l.trim_start();
            !(t.starts_with("//") || t.starts_with("/*") || t.starts_with("* ") || t == "*" || t.starts_with("*/"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn strip_quotes(value: &str) -> String {
    let value = value.trim();
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            return value[1..value.len() - 1].to_owned();
        }
    }
    value.to_owned()
}

/// Every double-quoted string on one line (no escape handling; the sources
/// parsed with this hold plain path and identifier literals).
fn quoted_strings(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('"') {
        let tail = &rest[start + 1..];
        match tail.find('"') {
            Some(end) => {
                out.push(tail[..end].to_owned());
                rest = &tail[end + 1..];
            }
            None => break,
        }
    }
    out
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '-')
}

// ==================================================================== gates ==

struct GateOutput {
    json: J,
    count: usize,
}

struct Script {
    name: String,
    cmd: String,
}

/// The `scripts` block of `package.json`, one `"name": "command"` per line.
fn parse_package_scripts(text: &str) -> Res<Vec<Script>> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if !inside {
            inside = line == "  \"scripts\": {";
            continue;
        }
        if line == "  }," || line == "  }" {
            return Ok(out);
        }
        let t = line.trim();
        let t = t.strip_prefix('"').ok_or("package.json scripts: unexpected line shape")?;
        let (name, rest) = t.split_once("\": \"").ok_or("package.json scripts: no value")?;
        let rest = rest.trim_end_matches(',');
        let rest = rest.strip_suffix('"').ok_or("package.json scripts: unterminated value")?;
        out.push(Script {
            name: name.to_owned(),
            cmd: rest.replace("\\\"", "\"").replace("\\\\", "\\"),
        });
    }
    Err("package.json: scripts block not found or not closed".into())
}

/// `bun run NAME` occurrences, in order.
fn bun_run_names(text: &str) -> Vec<String> {
    text.match_indices("bun run ")
        .filter_map(|(at, marker)| {
            let name: String = text[at + marker.len()..].chars().take_while(|c| is_name_char(*c)).collect();
            let name = name.trim_end_matches(':').to_owned();
            if name.is_empty() { None } else { Some(name) }
        })
        .collect()
}

/// `bun scripts/FILE` occurrences, as `scripts/FILE`.
fn bun_script_files(text: &str) -> Vec<String> {
    text.match_indices("bun scripts/")
        .map(|(at, _)| {
            let tail = &text[at + "bun ".len()..];
            let end = tail
                .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ';' | ')' | '`'))
                .unwrap_or(tail.len());
            tail[..end].to_owned()
        })
        .collect()
}

fn expand_script(name: &str, scripts: &BTreeMap<String, String>, seen: &mut BTreeSet<String>) {
    if !seen.insert(name.to_owned()) {
        return;
    }
    if let Some(cmd) = scripts.get(name) {
        for inner in bun_run_names(cmd) {
            expand_script(&inner, scripts, seen);
        }
    }
}

/// A script is a gate when its name or command says it verifies something.
fn gate_like_name(name: &str) -> bool {
    let segments: Vec<&str> = name.split(':').collect();
    segments.iter().any(|s| matches!(*s, "check" | "verify"))
        || segments.last().is_some_and(|s| matches!(*s, "lint" | "typecheck" | "test" | "e2e" | "coverage" | "build"))
}

fn gate_like_script(script: &Script) -> bool {
    gate_like_name(&script.name) || script.cmd.contains(" --check")
}

struct Job {
    id: String,
    name: String,
    runs: Vec<String>,
}

struct Workflow {
    file: String,
    name: String,
    events: Vec<String>,
    jobs: Vec<Job>,
}

impl Workflow {
    /// Runs on pull requests or branch pushes: a failure is visible before merge.
    fn pr_blocking(&self) -> bool {
        self.events.iter().any(|e| e == "pull_request" || e == "push:branches")
    }
}

fn parse_workflow(file: &str, text: &str) -> Workflow {
    let lines: Vec<&str> = text.lines().collect();
    let mut name = String::new();
    let mut raw_events: Vec<String> = Vec::new();
    let (mut push_tags, mut push_branches) = (false, false);
    let mut jobs: Vec<Job> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if line.starts_with("name:") && name.is_empty() {
            name = strip_quotes(&line[5..]);
        } else if let Some(inline) = line.strip_prefix("on:") {
            let inline = inline.trim();
            if inline.starts_with('[') {
                for event in inline.trim_matches(|c| c == '[' || c == ']').split(',') {
                    raw_events.push(event.trim().to_owned());
                }
            }
            let mut current = String::new();
            let mut next = index + 1;
            while next < lines.len() {
                let l = lines[next];
                let t = l.trim();
                if !t.is_empty() && !t.starts_with('#') && indent_of(l) == 0 {
                    break;
                }
                if indent_of(l) == 2 && !t.starts_with('#') && !t.is_empty() {
                    current = t.split(':').next().unwrap_or("").to_owned();
                    raw_events.push(current.clone());
                } else if indent_of(l) >= 4 && current == "push" {
                    push_tags |= t.starts_with("tags:");
                    push_branches |= t.starts_with("branches:");
                }
                next += 1;
            }
            index = next;
            continue;
        } else if line == "jobs:" {
            let mut next = index + 1;
            while next < lines.len() {
                let l = lines[next];
                let t = l.trim();
                if !t.is_empty() && !t.starts_with('#') && indent_of(l) == 0 {
                    break;
                }
                if indent_of(l) == 2 && t.ends_with(':') && !t.starts_with('#') {
                    jobs.push(Job { id: t.trim_end_matches(':').to_owned(), name: String::new(), runs: Vec::new() });
                } else if let Some(job) = jobs.last_mut() {
                    if indent_of(l) == 4 && t.starts_with("name:") && job.name.is_empty() {
                        job.name = strip_quotes(&t[5..]);
                    }
                    let step = t.strip_prefix("- ").unwrap_or(t);
                    if let Some(rest) = step.strip_prefix("run:") {
                        let key_indent = indent_of(l) + if t.starts_with("- ") { 2 } else { 0 };
                        let rest = rest.trim();
                        if rest.is_empty() || rest.starts_with('|') || rest.starts_with('>') {
                            let mut body = String::new();
                            let mut cursor = next + 1;
                            while cursor < lines.len() {
                                let b = lines[cursor];
                                if !b.trim().is_empty() && indent_of(b) <= key_indent {
                                    break;
                                }
                                if !b.trim_start().starts_with('#') {
                                    body.push_str(b.trim());
                                    body.push('\n');
                                }
                                cursor += 1;
                            }
                            job.runs.push(body);
                        } else {
                            job.runs.push(rest.to_owned());
                        }
                    }
                }
                next += 1;
            }
            index = next;
            continue;
        }
        index += 1;
    }
    let mut events = Vec::new();
    for event in raw_events {
        if event == "push" {
            if push_branches || !push_tags {
                events.push("push:branches".to_owned());
            }
            if push_tags {
                events.push("push:tags".to_owned());
            }
        } else if !event.is_empty() {
            events.push(event);
        }
    }
    events.sort();
    events.dedup();
    Workflow { file: file.to_owned(), name, events, jobs }
}

/// Docs that name checks: reference counts and first location per name.
struct DocRef {
    first_path: String,
    first_line: usize,
    count: usize,
}

fn build_gates(awcms: &Path) -> Res<GateOutput> {
    let package = read_text(&awcms.join("package.json"))?;
    let scripts = parse_package_scripts(&package)?;
    let script_map: BTreeMap<String, String> = scripts.iter().map(|s| (s.name.clone(), s.cmd.clone())).collect();
    if !script_map.contains_key("check") {
        return Err("package.json has no `check` script".into());
    }

    // Workflows (CI files), in name order.
    let workflow_dir = awcms.join(".github/workflows");
    let mut workflows: Vec<Workflow> = Vec::new();
    for file in list_files(&workflow_dir)? {
        if file.ends_with(".yml") || file.ends_with(".yaml") {
            let text = read_text(&workflow_dir.join(&file))?;
            workflows.push(parse_workflow(&format!(".github/workflows/{file}"), &text));
        }
    }
    let ci = workflows
        .iter()
        .find(|w| w.file.ends_with("/ci.yml"))
        .ok_or("AWCMS .github/workflows/ci.yml missing")?;
    let ci_runs_check = ci.jobs.iter().any(|j| j.runs.iter().any(|r| bun_run_names(r).iter().any(|n| n == "check")));
    if !ci_runs_check {
        return Err("AWCMS ci.yml no longer runs `bun run check`; gate derivation needs review".into());
    }

    let mut chain = BTreeSet::new();
    expand_script("check", &script_map, &mut chain);
    chain.remove("check");

    // For every workflow job: which scripts and files it reaches, directly or
    // through `bun run` chains.
    struct Reach {
        label: String,
        pr_blocking: bool,
        scripts: BTreeSet<String>,
    }
    let mut reaches: Vec<Reach> = Vec::new();
    let mut workflow_json = Vec::new();
    for wf in &workflows {
        let mut jobs_json = Vec::new();
        for job in &wf.jobs {
            let mut reached = BTreeSet::new();
            let mut files = BTreeSet::new();
            let mut runs_bun_test = false;
            for run in &job.runs {
                for name in bun_run_names(run) {
                    expand_script(&name, &script_map, &mut reached);
                }
                for file in bun_script_files(run) {
                    files.insert(file);
                }
                runs_bun_test |= run.contains("bun test");
            }
            for script in &scripts {
                let named_files = bun_script_files(&script.cmd);
                if named_files.iter().any(|f| files.contains(f)) {
                    expand_script(&script.name, &script_map, &mut reached);
                }
            }
            let label = format!("{}:{}", wf.file.trim_start_matches(".github/workflows/"), job.id);
            jobs_json.push(jo(vec![
                ("id", js(&job.id)),
                ("name", js(&job.name)),
                ("runs_check_chain", J::B(reached.contains("check"))),
                ("bun_test_direct", J::B(runs_bun_test)),
                ("scripts_reached", jn(reached.len())),
            ]));
            reaches.push(Reach { label, pr_blocking: wf.pr_blocking(), scripts: reached });
        }
        workflow_json.push(jo(vec![
            ("file", js(&wf.file)),
            ("name", js(&wf.name)),
            ("events", jstrs(&wf.events)),
            ("pr_blocking", J::B(wf.pr_blocking())),
            ("jobs", J::A(jobs_json)),
        ]));
    }

    let mut gates: Vec<(String, J, &'static str)> = Vec::new();

    // (a) gate-like package scripts.
    for script in scripts.iter().filter(|s| gate_like_script(s)) {
        let mut pr_jobs = Vec::new();
        let mut other_jobs = Vec::new();
        for reach in &reaches {
            if reach.scripts.contains(&script.name) {
                if reach.pr_blocking { pr_jobs.push(reach.label.clone()) } else { other_jobs.push(reach.label.clone()) }
            }
        }
        let script_file = bun_script_files(&script.cmd).into_iter().find(|f| awcms.join(f).is_file());
        let (status, reason) = if !pr_jobs.is_empty() {
            ("enforced", "run by a workflow job that triggers on pull_request or branch push".to_owned())
        } else if !other_jobs.is_empty() {
            ("advisory", "run only by workflow jobs that never trigger on pull_request or branch push".to_owned())
        } else {
            ("advisory", "package.json script exists; no workflow job runs it".to_owned())
        };
        let mut source = vec!["package.json".to_owned()];
        if let Some(f) = &script_file {
            source.push(f.clone());
        }
        source.extend(pr_jobs.iter().chain(other_jobs.iter()).map(|l| format!(".github/workflows/{l}")));
        gates.push((
            format!("script:{}", script.name),
            jo(vec![
                ("id", js(&format!("script:{}", script.name))),
                ("kind", js("package-script")),
                ("status", js(status)),
                ("reason", js(&reason)),
                ("command", js(&script.cmd)),
                ("script_file", jopt(script_file.as_deref())),
                ("in_check_chain", J::B(chain.contains(&script.name))),
                ("ci_jobs", jstrs(&pr_jobs)),
                ("other_jobs", jstrs(&other_jobs)),
                ("source", jstrs(&source)),
            ]),
            status,
        ));
    }

    // (b) workflow jobs that run on pull requests or branch pushes.
    for workflow in &workflows {
        if !workflow.pr_blocking() {
            continue;
        }
        for job in &workflow.jobs {
            let id = format!("job:{}:{}", workflow.file.trim_start_matches(".github/workflows/"), job.id);
            gates.push((
                id.clone(),
                jo(vec![
                    ("id", js(&id)),
                    ("kind", js("workflow-job")),
                    ("status", js("enforced")),
                    ("reason", js("job runs on pull_request or branch push")),
                    ("name", js(&job.name)),
                    ("events", jstrs(&workflow.events)),
                    ("source", jstrs(&[workflow.file.clone()])),
                ]),
                "enforced",
            ));
        }
    }

    // (c) check-like script files that nothing wires up.
    let all_run_text: String = workflows.iter().flat_map(|w| w.jobs.iter().flat_map(|j| j.runs.iter())).cloned().collect::<Vec<_>>().join("\n");
    let all_commands: String = scripts.iter().map(|s| s.cmd.as_str()).collect::<Vec<_>>().join("\n");
    for file in list_files(&awcms.join("scripts"))? {
        if file.contains('/') || !(file.ends_with(".ts") || file.ends_with(".mjs")) {
            continue;
        }
        let stem = file.rsplit_once('.').map_or(file.as_str(), |(s, _)| s);
        if !(stem.contains("check") || stem.contains("verify")) {
            continue;
        }
        let path = format!("scripts/{file}");
        if all_commands.contains(&path) || all_run_text.contains(&path) {
            continue;
        }
        let id = format!("file:{path}");
        gates.push((
            id.clone(),
            jo(vec![
                ("id", js(&id)),
                ("kind", js("script-file")),
                ("status", js("advisory")),
                ("reason", js("check-like script file exists; no package.json script or workflow step runs it")),
                ("source", jstrs(&[path])),
            ]),
            "advisory",
        ));
    }

    // (d) absent: checks that docs name as `bun run` or script files and that do not exist.
    let mut doc_files: Vec<String> = ["AGENTS.md", "README.md", "CONTRIBUTING.md"]
        .iter()
        .filter(|f| awcms.join(f).is_file())
        .map(|f| (*f).to_owned())
        .collect();
    for file in list_files(&awcms.join("docs"))? {
        if file.ends_with(".md") && !file.ends_with(".id.md") {
            doc_files.push(format!("docs/{file}"));
        }
    }
    doc_files.sort();
    let mut named_scripts: BTreeMap<String, DocRef> = BTreeMap::new();
    let mut named_files: BTreeMap<String, DocRef> = BTreeMap::new();
    for path in &doc_files {
        let text = read_text(&awcms.join(path))?;
        for (number, line) in text.lines().enumerate() {
            for name in bun_run_names(line) {
                if gate_like_name(&name) && !script_map.contains_key(&name) && !name.chars().any(|c| c.is_ascii_uppercase()) {
                    let entry = named_scripts.entry(name).or_insert_with(|| DocRef { first_path: path.clone(), first_line: number + 1, count: 0 });
                    entry.count += 1;
                }
            }
            for file in bun_script_files(line) {
                let stem = file.rsplit_once('.').map_or(file.as_str(), |(s, _)| s);
                let wanted = stem.contains("check") || stem.contains("verify");
                if wanted && !awcms.join(&file).is_file() && file.contains('.') {
                    let entry = named_files.entry(file).or_insert_with(|| DocRef { first_path: path.clone(), first_line: number + 1, count: 0 });
                    entry.count += 1;
                }
            }
        }
    }
    for (name, doc) in &named_scripts {
        let id = format!("doc-ref:{name}");
        gates.push((
            id.clone(),
            jo(vec![
                ("id", js(&id)),
                ("kind", js("documented-script-without-implementation")),
                ("status", js("absent")),
                ("reason", js("a doc names this check as `bun run`, and package.json has no such script")),
                ("doc_references", jn(doc.count)),
                ("source", jstrs(&[format!("{}:{}", doc.first_path, doc.first_line)])),
            ]),
            "absent",
        ));
    }
    for (file, doc) in &named_files {
        let id = format!("doc-ref-file:{file}");
        gates.push((
            id.clone(),
            jo(vec![
                ("id", js(&id)),
                ("kind", js("documented-script-file-without-implementation")),
                ("status", js("absent")),
                ("reason", js("a doc names this check script file, and it does not exist at this commit")),
                ("doc_references", jn(doc.count)),
                ("source", jstrs(&[format!("{}:{}", doc.first_path, doc.first_line)])),
            ]),
            "absent",
        ));
    }
    gates.sort_by(|a, b| a.0.cmp(&b.0));
    let count_of = |status: &str| gates.iter().filter(|g| g.2 == status).count();
    let (enforced, advisory, absent) = (count_of("enforced"), count_of("advisory"), count_of("absent"));
    let total = gates.len();
    let script_gates = scripts.iter().filter(|s| gate_like_script(s)).count();
    let json = jo(vec![
        ("source_sha", js(AWCMS_SHA)),
        ("method", js("statuses are derived from .github/workflows/*.yml, package.json scripts, scripts/ and docs; enforced = reached by a job triggered on pull_request or branch push; whether branch protection makes a job required is a repository setting outside the source tree")),
        (
            "summary",
            jo(vec![
                ("total", jn(total)),
                ("enforced", jn(enforced)),
                ("advisory", jn(advisory)),
                ("absent", jn(absent)),
                ("package_script_gates", jn(script_gates)),
                ("check_chain_length", jn(chain.len())),
                ("workflows", jn(workflows.len())),
            ]),
        ),
        ("workflows", J::A(workflow_json)),
        ("gates", J::A(gates.into_iter().map(|g| g.1).collect())),
    ]);
    Ok(GateOutput { json, count: total })
}

// =================================================================== routes ==

struct RouteOutput {
    json: J,
    files: usize,
    handlers: usize,
}

struct ModuleDesc {
    key: String,
    claims: Vec<String>,
    navigation: Vec<String>,
}

fn parse_module_descriptor(awcms: &Path, dir: &str, text: &str) -> Res<ModuleDesc> {
    let key_line = text
        .lines()
        .find(|l| l.starts_with("  key: "))
        .ok_or_else(|| format!("module {dir}: no top-level key"))?;
    let key_raw = key_line["  key: ".len()..].trim().trim_end_matches(',');
    let key = if key_raw.starts_with('"') {
        strip_quotes(key_raw)
    } else {
        let needle = format!("export const {key_raw} = \"");
        let module_dir = awcms.join("src/modules").join(dir);
        let mut found = None;
        for file in list_files(&module_dir)? {
            if file.ends_with(".ts") {
                let source = read_text(&module_dir.join(&file))?;
                if let Some(at) = source.find(&needle) {
                    let tail = &source[at + needle.len()..];
                    found = tail.find('"').map(|end| tail[..end].to_owned());
                    break;
                }
            }
        }
        found.ok_or_else(|| format!("module {dir}: key identifier {key_raw} unresolved"))?
    };

    let lines: Vec<&str> = text.lines().collect();
    let mut base_path = None;
    let mut routes: Vec<String> = Vec::new();
    if let Some(start) = lines.iter().position(|l| *l == "  api: {") {
        let mut in_routes = false;
        for line in &lines[start + 1..] {
            if *line == "  }," || *line == "  }" {
                break;
            }
            let t = line.trim();
            if in_routes {
                if t.starts_with(']') {
                    in_routes = false;
                } else if t.starts_with('"') {
                    routes.extend(quoted_strings(t).into_iter().take(1));
                }
            } else if let Some(rest) = t.strip_prefix("basePath:") {
                base_path = quoted_strings(rest).into_iter().next();
            } else if let Some(rest) = t.strip_prefix("routes:") {
                routes.extend(quoted_strings(rest));
                in_routes = !rest.contains(']');
            }
        }
    }
    if routes.is_empty() {
        routes.extend(base_path);
    }

    let mut navigation = Vec::new();
    if let Some(start) = lines.iter().position(|l| *l == "  navigation: [") {
        for line in &lines[start + 1..] {
            if line.starts_with("  ]") {
                break;
            }
            if let Some(at) = line.find("path: \"") {
                navigation.extend(quoted_strings(&line[at..]).into_iter().take(1));
            }
        }
    }
    Ok(ModuleDesc { key, claims: routes, navigation })
}

fn region<'a>(text: &'a str, start: &str, end: &str) -> Res<&'a str> {
    let from = text.find(start).ok_or_else(|| format!("marker not found: {start}"))?;
    let tail = &text[from..];
    let to = tail.find(end).ok_or_else(|| format!("marker not found: {end}"))?;
    Ok(&tail[..to])
}

struct FreeEndpoint {
    method: String,
    pattern: String,
    reason: String,
}

fn parse_session_free(text: &str) -> Res<Vec<FreeEndpoint>> {
    let block = region(text, "export const SESSION_FREE_BODY_ENDPOINTS", "const BODY_METHODS")?;
    let lines: Vec<&str> = block.lines().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let t = lines[index].trim();
        if let Some(rest) = t.strip_prefix("method:") {
            let method = quoted_strings(rest).into_iter().next().ok_or("session-free list: method")?;
            let pattern = lines
                .get(index + 1)
                .and_then(|l| l.trim().strip_prefix("pattern:"))
                .and_then(|r| quoted_strings(r).into_iter().next())
                .ok_or("session-free list: pattern must follow method")?;
            let mut reason = String::new();
            for look in index + 2..(index + 5).min(lines.len()) {
                let lt = lines[look].trim();
                if let Some(rest) = lt.strip_prefix("reason:") {
                    reason = quoted_strings(rest).into_iter().next().unwrap_or_default();
                    if reason.is_empty()
                        && let Some(next) = lines.get(look + 1)
                    {
                        reason = quoted_strings(next.trim()).into_iter().next().unwrap_or_default();
                    }
                    break;
                }
            }
            out.push(FreeEndpoint { method, pattern, reason });
            index += 2;
        } else {
            index += 1;
        }
    }
    if out.is_empty() {
        return Err("session-free endpoint list is empty".into());
    }
    Ok(out)
}

/// `:name` in a declared pattern and `[name]` in a route both match one segment.
fn pattern_matches_route(pattern: &str, route: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let route_parts: Vec<&str> = route.split('/').collect();
    pattern_parts.len() == route_parts.len()
        && pattern_parts.iter().zip(&route_parts).all(|(p, r)| {
            if p.starts_with(':') { r.starts_with('[') && r.ends_with(']') } else { p == r }
        })
}

/// `src/pages/blog/[tenantCode]/[slug].ts` -> `/blog/[tenantCode]/[slug]`.
fn route_of(relative_to_pages: &str) -> String {
    let mut route = relative_to_pages.to_owned();
    for ext in [".ts", ".astro", ".js"] {
        if let Some(stripped) = route.strip_suffix(ext) {
            route = stripped.to_owned();
            break;
        }
    }
    // Mirrors AWCMS `routeOf`: a nested `/index` collapses, the root keeps the
    // name `/index` (that is how PLATFORM_ROUTES spells the front door).
    if let Some(stripped) = route.strip_suffix("/index") {
        route = stripped.to_owned();
    }
    format!("/{}", route.trim_start_matches('/'))
}

const HTTP_METHODS: [&str; 8] = ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD", "ALL"];

/// Exported HTTP handlers of an Astro endpoint, with the text each one owns.
/// Text before the first handler (imports stripped) is shared by all of them.
fn split_handlers(text: &str) -> (Vec<(String, String)>, String) {
    let lines: Vec<&str> = text.lines().collect();
    let mut starts: Vec<(usize, String)> = Vec::new();
    for (number, line) in lines.iter().enumerate() {
        for prefix in ["export const ", "export async function ", "export function "] {
            if let Some(rest) = line.strip_prefix(prefix) {
                let name: String = rest.chars().take_while(char::is_ascii_alphabetic).collect();
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_uppercase()) && HTTP_METHODS.contains(&name.as_str()) {
                    starts.push((number, name));
                }
            }
        }
    }
    let first = starts.first().map_or(lines.len(), |s| s.0);
    let preamble_all = &lines[..first];
    let after_imports = preamble_all
        .iter()
        .rposition(|l| l.starts_with("} from \"") || (l.starts_with("import ") && l.contains(" from ")))
        .map_or(0, |p| p + 1);
    let preamble = preamble_all[after_imports..].join("\n");
    let mut handlers = Vec::new();
    for (position, (start, name)) in starts.iter().enumerate() {
        let end = starts.get(position + 1).map_or(lines.len(), |next| next.0);
        handlers.push((name.clone(), lines[*start..end].join("\n")));
    }
    (handlers, preamble)
}

struct OaOp {
    method: String,
    path: String,
    operation_id: String,
    security: &'static str,
    request_ref: Option<String>,
    has_request_body: bool,
    responses: Vec<String>,
}

fn ref_value(line: &str) -> Option<String> {
    let at = line.find("$ref:")?;
    Some(strip_quotes(line[at + 5..].trim()))
}

fn parse_openapi_ops(text: &str) -> Vec<OaOp> {
    let mut ops: Vec<OaOp> = Vec::new();
    let mut in_paths = false;
    let mut path = String::new();
    let (mut in_request, mut in_responses) = (false, false);
    let mut response_open: Option<usize> = None; // index into current op responses
    for line in text.lines() {
        if line == "paths:" {
            in_paths = true;
            continue;
        }
        if !in_paths {
            continue;
        }
        if !line.is_empty() && indent_of(line) == 0 {
            break;
        }
        let ind = indent_of(line);
        let t = line.trim();
        if ind == 2 && t.starts_with('/') && t.ends_with(':') {
            path = t.trim_end_matches(':').replace('{', "[").replace('}', "]");
            continue;
        }
        let method = t.trim_end_matches(':').to_ascii_uppercase();
        if ind == 4 && t.ends_with(':') && ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD"].contains(&method.as_str()) {
            ops.push(OaOp { method, path: path.clone(), operation_id: String::new(), security: "global", request_ref: None, has_request_body: false, responses: Vec::new() });
            in_request = false;
            in_responses = false;
            response_open = None;
            continue;
        }
        let Some(op) = ops.last_mut() else { continue };
        if ind == 6 {
            in_request = t == "requestBody:";
            in_responses = t == "responses:";
            response_open = None;
            if in_request {
                op.has_request_body = true;
            }
            if let Some(id) = t.strip_prefix("operationId:") {
                op.operation_id = strip_quotes(id);
            } else if t == "security: []" {
                op.security = "none";
            } else if t == "security:" {
                op.security = "declared";
            }
            continue;
        }
        if in_request && op.request_ref.is_none() && t.contains("$ref:") {
            op.request_ref = ref_value(t);
        } else if in_responses {
            if ind == 8 && t.ends_with(':') {
                let code = strip_quotes(t.trim_end_matches(':'));
                op.responses.push(format!("{code} -"));
                response_open = Some(op.responses.len() - 1);
            } else if let Some(open) = response_open
                && let Some(reference) = ref_value(t)
                && op.responses[open].ends_with(" -")
            {
                let code = op.responses[open].trim_end_matches(" -").to_owned();
                op.responses[open] = format!("{code} {reference}");
            }
        }
    }
    ops
}

fn build_routes(awcms: &Path) -> Res<RouteOutput> {
    // Ownership sources.
    let module_root = awcms.join("src/modules");
    let mut modules: Vec<ModuleDesc> = Vec::new();
    for dir in list_dirs(&module_root)? {
        if dir == "_shared" {
            continue;
        }
        let module_file = module_root.join(&dir).join("module.ts");
        if module_file.is_file() {
            modules.push(parse_module_descriptor(awcms, &dir, &read_text(&module_file)?)?);
        }
    }
    ensure_count("route owner modules", modules.len(), EXPECTED_MODULES)?;
    let mut claims: Vec<(String, String)> = Vec::new();
    for module in &modules {
        for prefix in &module.claims {
            claims.push((prefix.clone(), module.key.clone()));
        }
    }
    claims.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
    let mut navigation: Vec<(String, String)> = Vec::new();
    for module in &modules {
        for path in &module.navigation {
            navigation.push((path.clone(), module.key.clone()));
        }
    }
    let sidebar = read_text(&awcms.join("src/modules/module-management/domain/sidebar-menu.ts"))?;
    for line in region(&sidebar, "export const CORE_NAV_ENTRIES", "];")?.lines() {
        if let Some(at) = line.find("path: \"") {
            for path in quoted_strings(&line[at..]).into_iter().take(1) {
                navigation.push((path, "core".to_owned()));
            }
        }
    }
    let platform_text = read_text(&awcms.join("scripts/validate-module-routes.ts"))?;
    let platform: BTreeSet<String> = region(&platform_text, "const PLATFORM_ROUTES", "];")?
        .lines()
        .filter_map(|l| l.trim().strip_prefix("route:").and_then(|r| quoted_strings(r).into_iter().next()))
        .collect();
    let middleware = read_text(&awcms.join("src/middleware.ts"))?;
    if !middleware.contains("const PROTECTED_PREFIX = \"/admin\";") {
        return Err("src/middleware.ts no longer declares PROTECTED_PREFIX = \"/admin\"".into());
    }
    let free = parse_session_free(&read_text(&awcms.join("src/lib/security/api-body-auth-boundary.ts"))?)?;
    let openapi = parse_openapi_ops(&read_text(&awcms.join("openapi/awcms-public-api.openapi.yaml"))?);
    let oa_index: BTreeMap<(String, String), usize> =
        openapi.iter().enumerate().map(|(i, o)| ((o.method.clone(), o.path.clone()), i)).collect();

    // Walk the pages.
    let pages_root = awcms.join("src/pages");
    let files = list_files(&pages_root)?;
    let mut surface_counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut entries: Vec<J> = Vec::new();
    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut owner_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut agreement_counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut matched_oa: BTreeSet<usize> = BTreeSet::new();
    for relative in &files {
        let route = route_of(relative);
        let shown_route = if route == "/index" { "/".to_owned() } else { route.clone() };
        let surface = if route.starts_with("/api/") {
            "api"
        } else if route == "/admin" || route.starts_with("/admin/") {
            "admin"
        } else {
            "public"
        };
        *surface_counts.entry(surface).or_default() += 1;
        let file_path = format!("src/pages/{relative}");
        // Markers are searched in code only: a comment that names a function
        // is not a call to it.
        let text = strip_comment_lines(&read_text(&pages_root.join(relative))?);

        // Owner.
        let (owner, basis) = if surface == "admin" {
            resolve_admin_owner(&route, &navigation)
                .ok_or_else(|| format!("{file_path}: no navigation entry owns {route}"))?
        } else if platform.contains(&route) {
            ("platform".to_owned(), "PLATFORM_ROUTES".to_owned())
        } else {
            let claim = claims
                .iter()
                .find(|(prefix, _)| route.starts_with(prefix.as_str()))
                .ok_or_else(|| format!("{file_path}: no module claims {route}"))?;
            (claim.1.clone(), format!("api.routes:{}", claim.0))
        };

        // Handlers.
        let (handlers, preamble) = if relative.ends_with(".astro") {
            (vec![("GET".to_owned(), text.clone())], String::new())
        } else {
            split_handlers(&text)
        };
        if handlers.is_empty() {
            return Err(format!("{file_path}: no exported HTTP handler found").into());
        }
        let is_page = relative.ends_with(".astro");
        let imports_session =
            !is_page && (text.contains("lib/auth/session-token") || text.contains("lib/auth/ssr-session"));
        for (method, body) in &handlers {
            let scope = format!("{body}\n{preamble}");
            let has = |needle: &str| scope.contains(needle);
            let free_match = free.iter().find(|f| f.method == *method && pattern_matches_route(&f.pattern, &route));
            let mut markers: Vec<String> = Vec::new();
            let mut note = String::new();
            let class;
            let mut machine = "not-applicable";
            if surface == "admin" {
                class = "admin";
                markers.push("middleware:PROTECTED_PREFIX=/admin".to_owned());
            } else if let Some(f) = free_match {
                markers.push(format!("SESSION_FREE_BODY_ENDPOINTS:{} {}", f.method, f.pattern));
                note.clone_from(&f.reason);
                if has("verifySyncHeaders") {
                    class = "machine-credential";
                    markers.push("verifySyncHeaders".to_owned());
                } else if has("defineClientCredentialTenantRoute") {
                    class = "machine-credential";
                    markers.push("defineClientCredentialTenantRoute".to_owned());
                } else {
                    class = "public";
                }
            } else if has("verifySyncHeaders") || has("defineClientCredentialTenantRoute") {
                class = "machine-credential";
                markers.push(if has("verifySyncHeaders") { "verifySyncHeaders" } else { "defineClientCredentialTenantRoute" }.to_owned());
            } else if has("authorizeInTransaction(") || has("defineTenantRoute") || has("defineSseTenantRoute") || has("defineSelfServiceTenantRoute") {
                class = "session";
                for marker in ["authorizeInTransaction(", "defineTenantRoute", "defineSseTenantRoute", "defineSelfServiceTenantRoute"] {
                    if has(marker) {
                        markers.push(marker.trim_end_matches('(').to_owned());
                    }
                }
                machine = if has("isMachineCredentialToken(") {
                    markers.push("isMachineCredentialToken".to_owned());
                    "refused-in-handler"
                } else if has("authorizeInTransaction(") || has("defineTenantRoute") || has("defineSseTenantRoute") {
                    "accepted-by-guard"
                } else {
                    "unspecified"
                };
            } else if (has("resolveAuthInputs(") || imports_session) && (has("\"AUTH_REQUIRED\"") || has("fail(401")) {
                class = "session";
                markers.push("hand-written handler answers 401 AUTH_REQUIRED without a credential".to_owned());
                machine = if has("isMachineCredentialToken(") { "refused-in-handler" } else { "unspecified" };
            } else if imports_session {
                class = "indeterminate";
                markers.push("session-module-import-without-authorization-call".to_owned());
            } else {
                class = "public";
                markers.push("no-credential-marker".to_owned());
            }

            // OpenAPI cross-check (API surface only).
            let (oa_security, agreement) = if surface == "api" {
                match oa_index.get(&(method.clone(), route.clone())) {
                    Some(i) => {
                        matched_oa.insert(*i);
                        let security = openapi[*i].security;
                        let agrees = match class {
                            "public" => security == "none",
                            "session" | "machine-credential" => security != "none",
                            _ => false,
                        };
                        let verdict = if class == "indeterminate" { "indeterminate" } else if agrees { "agrees" } else { "differs" };
                        (security, verdict)
                    }
                    None => ("undocumented", "not-in-openapi"),
                }
            } else {
                ("not-applicable", "not-applicable")
            };
            *class_counts.entry(class.to_owned()).or_default() += 1;
            *owner_counts.entry(owner.clone()).or_default() += 1;
            *agreement_counts.entry(agreement).or_default() += 1;
            let mut fields = vec![
                ("file", js(&file_path)),
                ("route", js(&shown_route)),
                ("method", js(if relative.ends_with(".astro") { "GET" } else { method })),
                ("kind", js(if relative.ends_with(".astro") { "page" } else { "endpoint" })),
                ("surface", js(surface)),
                ("owner", js(&owner)),
                ("owner_basis", js(&basis)),
                ("auth_class", js(class)),
                ("auth_markers", jstrs(&markers)),
                ("machine_credential", js(machine)),
                ("openapi_security", js(oa_security)),
                ("openapi_agreement", js(agreement)),
            ];
            if !note.is_empty() {
                fields.push(("note", js(&note)));
            }
            entries.push(jo(fields));
        }
    }

    for (surface, expected) in EXPECTED_ROUTE_FILES {
        ensure_count(&format!("route files ({surface})"), surface_counts.get(surface).copied().unwrap_or(0), expected)?;
    }
    let unmatched_oa = openapi.len() - matched_oa.len();
    if unmatched_oa != 0 {
        let missing: Vec<String> = openapi
            .iter()
            .enumerate()
            .filter(|(i, _)| !matched_oa.contains(i))
            .map(|(_, o)| format!("{} {}", o.method, o.path))
            .collect();
        return Err(format!("OpenAPI operations without a route handler: {}", missing.join(", ")).into());
    }
    let handlers = entries.len();
    let summary_counts = |counts: &BTreeMap<String, usize>| J::O(counts.iter().map(|(k, v)| (k.clone(), jn(*v))).collect());
    let json = jo(vec![
        ("source_sha", js(AWCMS_SHA)),
        ("method", js("routes = every file under src/pages; handlers = exported HTTP method handlers (.astro pages count as GET); owner from module api.routes (longest prefix), PLATFORM_ROUTES, module navigation and CORE_NAV_ENTRIES; auth_class from code markers, cross-checked against OpenAPI security; indeterminate = a session module is imported but no authorization call was found")),
        (
            "summary",
            jo(vec![
                ("route_files", jn(files.len())),
                ("api_route_files", jn(surface_counts.get("api").copied().unwrap_or(0))),
                ("admin_route_files", jn(surface_counts.get("admin").copied().unwrap_or(0))),
                ("public_route_files", jn(surface_counts.get("public").copied().unwrap_or(0))),
                ("handlers", jn(handlers)),
                ("openapi_operations", jn(openapi.len())),
                ("auth_class", summary_counts(&class_counts)),
                ("owner", summary_counts(&owner_counts)),
                ("openapi_agreement", J::O(agreement_counts.iter().map(|(k, v)| ((*k).to_owned(), jn(*v))).collect())),
            ]),
        ),
        ("routes", J::A(entries)),
    ]);
    Ok(RouteOutput { json, files: files.len(), handlers })
}

fn list_dirs(root: &Path) -> Res<Vec<String>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            out.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    out.sort();
    Ok(out)
}

/// Exact navigation path, else the page above a dynamic segment.
fn resolve_admin_owner(route: &str, navigation: &[(String, String)]) -> Option<(String, String)> {
    let exact = |path: &str| navigation.iter().find(|(p, _)| p == path);
    if let Some((path, owner)) = exact(route) {
        return Some((owner.clone(), format!("navigation:{path}")));
    }
    let dynamic_at = route.find("/[")?;
    let parent = &route[..dynamic_at];
    exact(parent).map(|(path, owner)| (owner.clone(), format!("dynamic-parent:{path}")))
}

// ============================================================== behaviour ==

struct BehaviourOutput {
    json: J,
    operations: usize,
    exchanges: usize,
    frozen_tests: Vec<String>,
}

struct Exchange {
    file: String,
    line: usize,
    method: String,
    path: String,
    test: String,
    context: String,
    origin: &'static str,
    statuses: Vec<u16>,
    codes: Vec<String>,
}

/// First `METHOD /path` in a title.
fn parse_operation(title: &str) -> Option<(String, String)> {
    let mut best: Option<(usize, String, String)> = None;
    for method in ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"] {
        let needle = format!("{method} /");
        for (at, _) in title.match_indices(&needle) {
            let boundary = title[..at].chars().next_back().is_none_or(|c| !c.is_ascii_alphanumeric());
            if !boundary {
                continue;
            }
            let tail = &title[at + method.len() + 1..];
            let end = tail
                .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | '`' | ')' | ',' | ';'))
                .unwrap_or(tail.len());
            let path = tail[..end].trim_end_matches(['.', ':']).to_owned();
            if best.as_ref().is_none_or(|(p, _, _)| at < *p) {
                best = Some((at, method.to_owned(), path));
            }
            break;
        }
    }
    best.map(|(_, m, p)| (m, p))
}

fn call_title(lines: &[&str], index: usize, prefixes: &[&str]) -> Option<String> {
    let t = lines[index].trim_start();
    let prefix = prefixes.iter().find(|p| t.starts_with(**p))?;
    let mut rest = t[prefix.len()..].trim_start();
    if rest.is_empty() {
        rest = lines.get(index + 1)?.trim_start();
    }
    let quote = rest.chars().next().filter(|c| matches!(c, '"' | '\'' | '`'))?;
    let body = &rest[1..];
    let mut title = String::new();
    let mut escaped = false;
    for ch in body.chars() {
        if escaped {
            title.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return Some(title);
        } else {
            title.push(ch);
        }
    }
    None
}

/// `new Request("https://host/api/...", { method: "X" ...` inside a test block.
fn request_literal(block: &[&str]) -> Option<(String, String)> {
    for (offset, line) in block.iter().enumerate() {
        let Some(at) = line.find("new Request(\"") else { continue };
        let url = quoted_strings(&line[at..]).into_iter().next()?;
        let path_at = url.find("/api/")?;
        let path = url[path_at..].split(['?', '#']).next()?.to_owned();
        let window = block[offset..(offset + 4).min(block.len())].join(" ");
        let method = window
            .find("method: \"")
            .and_then(|m| quoted_strings(&window[m..]).into_iter().next())
            .unwrap_or_else(|| "GET".to_owned());
        return Some((method.to_ascii_uppercase(), path));
    }
    None
}

const DESCRIBE_CALLS: [&str; 2] = ["describe(", "describe.skip("];
const TEST_CALLS: [&str; 4] = ["test(", "it(", "test.skip(", "it.skip("];

fn extract_exchanges(file: &str, text: &str) -> Vec<Exchange> {
    let lines: Vec<&str> = text.lines().collect();
    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut out = Vec::new();
    let is_call = |line: &str| DESCRIBE_CALLS.iter().chain(TEST_CALLS.iter()).any(|p| line.trim_start().starts_with(p));
    for index in 0..lines.len() {
        let ind = indent_of(lines[index]);
        if let Some(title) = call_title(&lines, index, &DESCRIBE_CALLS) {
            while stack.last().is_some_and(|(i, _)| *i >= ind) {
                stack.pop();
            }
            stack.push((ind, title));
        } else if let Some(title) = call_title(&lines, index, &TEST_CALLS) {
            while stack.last().is_some_and(|(i, _)| *i >= ind) {
                stack.pop();
            }
            let mut end = index + 1;
            while end < lines.len() && !(is_call(lines[end]) && indent_of(lines[end]) <= ind) {
                end += 1;
            }
            let block = lines[index..end].join("\n");
            let mut statuses: Vec<u16> = Vec::new();
            for marker in [".status).toBe(", ".status).toEqual("] {
                for (at, _) in block.match_indices(marker) {
                    let digits: String = block[at + marker.len()..].chars().take(3).collect();
                    if digits.len() == 3
                        && block[at + marker.len() + 3..].starts_with(')')
                        && let Ok(code) = digits.parse::<u16>()
                        && !statuses.contains(&code)
                    {
                        statuses.push(code);
                    }
                }
            }
            if statuses.is_empty() {
                continue;
            }
            let mut codes: Vec<String> = Vec::new();
            for (at, marker) in block.match_indices("code).toBe(\"") {
                let tail = &block[at + marker.len()..];
                if let Some(end_quote) = tail.find('"') {
                    let code = tail[..end_quote].to_owned();
                    if !codes.contains(&code) {
                        codes.push(code);
                    }
                }
            }
            let (operation, context, origin) = match parse_operation(&title) {
                Some(op) => (Some(op), String::new(), "test-title"),
                None => match stack.iter().rev().find_map(|(_, t)| parse_operation(t).map(|op| (op, t.clone()))) {
                    Some((op, ctx)) => (Some(op), ctx, "describe-title"),
                    None => (request_literal(&lines[index..end]), String::new(), "request-literal"),
                },
            };
            if let Some((method, path)) = operation {
                out.push(Exchange { file: file.to_owned(), line: index + 1, method, path, test: title, context, origin, statuses, codes });
            }
        }
    }
    out
}

fn build_behaviour(awcms: &Path, astro: &Path) -> Res<BehaviourOutput> {
    let openapi_text = read_text(&awcms.join("openapi/awcms-public-api.openapi.yaml"))?;
    let operations = parse_openapi_ops(&openapi_text);
    let scalar_examples = openapi_text.lines().filter(|l| l.trim_start().starts_with("example:")).count();
    let block_examples = openapi_text.lines().filter(|l| l.trim_start().starts_with("examples:")).count();

    let mut exchanges: Vec<Exchange> = Vec::new();
    for file in list_files(&awcms.join("tests"))? {
        if !file.ends_with(".test.ts") {
            continue;
        }
        let relative = format!("tests/{file}");
        exchanges.extend(extract_exchanges(&relative, &read_text(&awcms.join(&relative))?));
    }
    let astro_test = read_text(&astro.join("tests/kontrak-awcms.test.mjs"))?;
    let astro_mocks = astro_test.matches("globalThis.fetch = ").count();

    let mut frozen: BTreeSet<String> = VECTOR_TEST_SOURCES.iter().map(|s| (*s).to_owned()).collect();
    for exchange in &exchanges {
        frozen.insert(exchange.file.clone());
    }
    for relative in &frozen {
        if !awcms.join(relative).is_file() {
            return Err(format!("cited test file missing at the pinned SHA: {relative}").into());
        }
    }

    let operation_json: Vec<J> = operations
        .iter()
        .map(|o| {
            jo(vec![
                ("method", js(&o.method)),
                ("path", js(&o.path)),
                ("operation_id", js(&o.operation_id)),
                ("security", js(o.security)),
                ("request_body", J::B(o.has_request_body)),
                ("request_schema_ref", jopt(o.request_ref.as_deref())),
                ("responses", jstrs(&o.responses)),
            ])
        })
        .collect();
    let exchange_json: Vec<J> = exchanges
        .iter()
        .map(|e| {
            jo(vec![
                ("method", js(&e.method)),
                ("path", js(&e.path)),
                ("test", js(&e.test)),
                ("operation_source", js(e.origin)),
                ("context", js(&e.context)),
                ("expected_status", J::A(e.statuses.iter().map(|s| jn(usize::from(*s))).collect())),
                ("expected_error_codes", jstrs(&e.codes)),
                ("source", js(&format!("ahliweb/awcms:{}:{}", e.file, e.line))),
                ("frozen_copy", js(&format!("{FROZEN_TEST_ROOT}/{}", e.file))),
            ])
        })
        .collect();
    let frozen_list: Vec<String> = frozen.into_iter().collect();
    let classes = vec![
        jo(vec![
            ("class", js("openapi-operation-contracts")),
            ("status", js("available")),
            ("count", jn(operations.len())),
            ("what", js("per operation: security, whether a request body is declared and its first schema $ref, and each documented response status with its $ref; copied from the frozen OpenAPI, not invented")),
        ]),
        jo(vec![
            ("class", js("test-asserted-exchanges")),
            ("status", js("available")),
            ("count", jn(exchanges.len())),
            ("what", js("AWCMS test cases whose title or enclosing describe names `METHOD /path` and whose body asserts a literal `.status` (and optionally an error `code`); the test files are frozen so each citation resolves in the corpus")),
        ]),
        jo(vec![
            ("class", js("astro-consumer-mocked-fetch")),
            ("status", js("available-by-reference")),
            ("count", jn(astro_mocks)),
            ("what", js("AWCMS-Astro contract test installs this many `globalThis.fetch` mocks of AWCMS responses; the file is already frozen as source/awcms-astro-contract.test.mjs and is not decomposed into fixtures")),
        ]),
        jo(vec![
            ("class", js("openapi-request-response-body-examples")),
            ("status", js("not-available")),
            ("count", jn(block_examples)),
            ("what", js(&format!("the bundled OpenAPI holds {scalar_examples} scalar property `example:` values and {block_examples} `examples:` blocks, so no request or response body example exists to freeze"))),
        ]),
        jo(vec![
            ("class", js("recorded-live-responses")),
            ("status", js("not-available")),
            ("count", jn(0)),
            ("what", js("the pinned source tree holds no captured HTTP traffic or golden response files; recording them needs a running AWCMS instance and is not done here")),
        ]),
    ];
    let json = jo(vec![
        ("source_sha", js(AWCMS_SHA)),
        ("astro_source_sha", js(ASTRO_SHA)),
        ("classes", J::A(classes)),
        ("frozen_test_sources", jstrs(&frozen_list)),
        ("openapi_operations", J::A(operation_json)),
        ("test_asserted_exchanges", J::A(exchange_json)),
    ]);
    Ok(BehaviourOutput { json, operations: operations.len(), exchanges: exchanges.len(), frozen_tests: frozen_list })
}

// ====================================================== implementation facts ==
//
// `rate-limits.json` and `credential-formats.json` are derived from the pinned
// AWCMS implementation, not from prose. Every fact cites `path`, `line` and an
// `anchor` (a substring of that line); the generator refuses to emit a fact
// whose anchor is not on the cited line, and the files it cites are copied to
// `source/awcms-impl/` so the corpus checker can re-resolve every citation.
// No credential value is ever read into an artifact: only names, formats and
// parameters.

/// Frozen copies of the implementing AWCMS files keep their repository-relative
/// path under this directory (for example `src/lib/security/rate-limit.ts`).
const FROZEN_IMPL_ROOT: &str = "source/awcms-impl";

type Files = BTreeMap<String, Vec<String>>;

/// Every `.ts` / `.astro` file under `src/` and every `.sql` file under `sql/`.
fn load_impl_files(awcms: &Path) -> Res<Files> {
    let mut files = Files::new();
    for (dir, extensions) in [("src", &[".ts", ".astro"][..]), ("sql", &[".sql"][..])] {
        for relative in list_files(&awcms.join(dir))? {
            if extensions.iter().any(|e| relative.ends_with(e)) {
                let path = format!("{dir}/{relative}");
                let text = read_text(&awcms.join(&path))?;
                files.insert(path, text.lines().map(str::to_owned).collect());
            }
        }
    }
    Ok(files)
}

struct Book<'a> {
    files: &'a Files,
    used: RefCell<BTreeSet<String>>,
}

impl<'a> Book<'a> {
    fn new(files: &'a Files) -> Self {
        Book { files, used: RefCell::new(BTreeSet::new()) }
    }

    fn lines(&self, rel: &str) -> Res<&'a Vec<String>> {
        self.files.get(rel).ok_or_else(|| format!("implementation file not found at the pinned SHA: {rel}").into())
    }

    /// A citation. The anchor must be on the cited line (1-based).
    fn fact(&self, rel: &str, line: usize, anchor: &str) -> Res<J> {
        let lines = self.lines(rel)?;
        let text = line.checked_sub(1).and_then(|i| lines.get(i)).ok_or_else(|| format!("{rel}:{line}: no such line"))?;
        if anchor.is_empty() || !text.contains(anchor) {
            return Err(format!("{rel}:{line}: anchor `{anchor}` is not on the cited line").into());
        }
        self.used.borrow_mut().insert(rel.to_owned());
        Ok(jo(vec![("path", js(rel)), ("line", jn(line)), ("anchor", js(anchor))]))
    }

    /// First line (1-based) at or after `from` containing `needle`.
    fn find_from(&self, rel: &str, needle: &str, from: usize) -> Res<usize> {
        let lines = self.lines(rel)?;
        lines
            .iter()
            .enumerate()
            .skip(from.saturating_sub(1))
            .find(|(_, l)| l.contains(needle))
            .map(|(i, _)| i + 1)
            .ok_or_else(|| format!("{rel}: `{needle}` not found").into())
    }

    fn find(&self, rel: &str, needle: &str) -> Res<usize> {
        self.find_from(rel, needle, 1)
    }

    fn fact_find(&self, rel: &str, needle: &str) -> Res<J> {
        let line = self.find(rel, needle)?;
        self.fact(rel, line, needle)
    }

    fn fact_find_from(&self, rel: &str, needle: &str, from: usize) -> Res<J> {
        let line = self.find_from(rel, needle, from)?;
        self.fact(rel, line, needle)
    }

    fn used_files(&self) -> Vec<String> {
        self.used.borrow().iter().cloned().collect()
    }
}

fn is_code_line(line: &str) -> bool {
    let t = line.trim_start();
    !(t.starts_with("//") || t.starts_with('*') || t.starts_with("/*"))
}

fn is_ident(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Collapses whitespace so multi-line expressions compare as one line.
fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").trim_end_matches(',').trim().to_owned()
}

/// `60 * 60 * 1000`, `50_000`, `5` -> value; anything else -> `None`.
fn lit_product(s: &str) -> Option<i64> {
    let mut acc: i64 = 1;
    for part in s.split('*') {
        let p = part.trim().replace('_', "");
        if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        acc = acc.checked_mul(p.parse::<i64>().ok()?)?;
    }
    Some(acc)
}

#[derive(Clone, Copy, PartialEq)]
enum Stop {
    /// Through the delimiter that closes the one at the start column.
    Group,
    /// Up to (excluding) the next `;` at depth zero.
    Semicolon,
}

/// Text starting at (`line0`, `col`), 0-based, with line comments dropped and
/// string / template literals kept whole. Newlines are kept so the span can be
/// mapped back to source lines.
fn scan(lines: &[String], line0: usize, col: usize, stop: Stop) -> Res<String> {
    let mut out = String::new();
    let mut depth: i32 = 0;
    let mut quote: Option<char> = None;
    for (offset, line) in lines.iter().enumerate().skip(line0) {
        let slice: &str = if offset == line0 { line.get(col..).ok_or("scan: bad column")? } else { line };
        let chars: Vec<char> = slice.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if let Some(q) = quote {
                out.push(c);
                if c == '\\'
                    && let Some(next) = chars.get(i + 1)
                {
                    out.push(*next);
                    i += 2;
                    continue;
                }
                if c == q {
                    quote = None;
                }
                i += 1;
                continue;
            }
            if c == '/' && chars.get(i + 1) == Some(&'/') {
                break;
            }
            match c {
                '"' | '\'' | '`' => quote = Some(c),
                '(' | '{' | '[' => depth += 1,
                ')' | '}' | ']' => depth -= 1,
                ';' if stop == Stop::Semicolon && depth == 0 => return Ok(out),
                _ => {}
            }
            out.push(c);
            if stop == Stop::Group && depth == 0 && matches!(c, ')' | '}' | ']') {
                return Ok(out);
            }
            i += 1;
        }
        out.push('\n');
        if quote.is_some_and(|q| q != '`') {
            quote = None;
        }
    }
    Err("scan: unterminated expression".into())
}

/// Byte positions of `sep` at nesting depth zero outside string literals.
fn top_positions(text: &str, sep: char) -> Vec<usize> {
    let mut found = Vec::new();
    let mut depth: i32 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (pos, c) in text.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' | '`' => quote = Some(c),
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            c if c == sep && depth == 0 => found.push(pos),
            _ => {}
        }
    }
    found
}

fn split_top(text: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut from = 0;
    for pos in top_positions(text, sep) {
        parts.push(text[from..pos].trim().to_owned());
        from = pos + sep.len_utf8();
    }
    let tail = text[from..].trim();
    if !tail.is_empty() {
        parts.push(tail.to_owned());
    }
    parts
}

fn split_pair(text: &str, sep: char) -> Option<(String, String)> {
    let pos = *top_positions(text, sep).first()?;
    Some((text[..pos].trim().to_owned(), text[pos + sep.len_utf8()..].trim().to_owned()))
}

/// Inside of a `( ... )`, `{ ... }` or `[ ... ]` group, outer delimiters removed.
fn group_inner(text: &str) -> &str {
    let t = text.trim();
    if t.len() >= 2 { t[1..t.len() - 1].trim() } else { "" }
}

fn strip_outer_parens(expr: &str) -> String {
    let mut e = expr.trim().to_owned();
    while e.starts_with('(') && e.ends_with(')') {
        let mut depth = 0;
        let mut closes_at_end = true;
        for (i, c) in e.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && i + 1 != e.len() {
                        closes_at_end = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !closes_at_end {
            break;
        }
        e = e[1..e.len() - 1].trim().to_owned();
    }
    e
}

/// The balanced group that follows `needle` on line `line0` (0-based), plus the
/// number of source lines it spans.
fn call_group(lines: &[String], line0: usize, needle: &str) -> Res<(String, usize)> {
    let line = &lines[line0];
    let at = line.find(needle).ok_or("call_group: needle not on line")?;
    let open = at + needle.len();
    let text = scan(lines, line0, open, Stop::Group)?;
    let span = text.matches('\n').count();
    Ok((text, span))
}

/// The statement `const NAME ... = <init>;` -> (1-based line, init text).
fn decl_init(book: &Book, rel: &str, name: &str) -> Res<Option<(usize, String)>> {
    let lines = book.lines(rel)?;
    for (index, line) in lines.iter().enumerate() {
        let t = line.trim_start();
        let rest = t.strip_prefix("export ").unwrap_or(t);
        let Some(after) = rest.strip_prefix("const ") else { continue };
        let Some(tail) = after.strip_prefix(name) else { continue };
        if !(tail.starts_with(' ') || tail.starts_with(':') || tail.starts_with('=')) {
            continue;
        }
        let name_at = line.find(&format!("const {name}")).ok_or("decl_init: name")? + "const ".len() + name.len();
        let eq = line[name_at..]
            .char_indices()
            .find(|(i, c)| {
                *c == '=' && !matches!(line[name_at + i + 1..].chars().next(), Some('>' | '='))
            })
            .map(|(i, _)| name_at + i + 1);
        let Some(eq) = eq else { continue };
        let init = scan(lines, index, eq, Stop::Semicolon)?;
        return Ok(Some((index + 1, norm(&init))));
    }
    Ok(None)
}

fn const_num(book: &Book, rel: &str, name: &str) -> Res<(usize, i64, J)> {
    let (line, init) = decl_init(book, rel, name)?.ok_or_else(|| format!("{rel}: const {name} not found"))?;
    let value = lit_product(&init).ok_or_else(|| format!("{rel}: const {name} is not a numeric literal"))?;
    let fact = book.fact(rel, line, &format!("const {name}"))?;
    Ok((line, value, fact))
}

fn const_str(book: &Book, rel: &str, name: &str) -> Res<(String, J)> {
    let (line, init) = decl_init(book, rel, name)?.ok_or_else(|| format!("{rel}: const {name} not found"))?;
    let value = quoted_strings(&init.replace('\'', "\"")).into_iter().next().ok_or_else(|| format!("{rel}: const {name} is not a string literal"))?;
    Ok((value, book.fact(rel, line, &format!("const {name}"))?))
}

// ------------------------------------------------------- numeric settings ----

/// How an operator-set number reaches a limiter, derived from the code that
/// declares it. `parsing` names the parser; `fallback` is the value used when
/// a guard rejects the input.
struct Val {
    env: Option<String>,
    default: i64,
    parsing: &'static str,
    fallback: Option<i64>,
    decls: Vec<J>,
}

impl Val {
    fn literal(value: i64) -> Val {
        Val { env: None, default: value, parsing: "literal", fallback: None, decls: Vec::new() }
    }
}

fn parsing_behaviour(parsing: &str) -> &'static str {
    match parsing {
        "literal" => "fixed in source; not operator-configurable",
        "parsePositiveIntSetting" | "parsePositiveIntEnv" => {
            "an absent or empty value uses the default; a non-numeric, non-integer, zero or negative value falls back to the default (warned once)"
        }
        "Number-coercion-guarded" => "a non-finite or non-positive result falls back to the stated fallback; an empty string coerces to 0 and therefore also falls back",
        "Number-coercion-unguarded" => "no validation: Number() is used as given, so a non-numeric value yields NaN and an empty string yields 0",
        "Number-env-guarded-in-function" => "an absent, non-numeric or non-positive value falls back to the default",
        "parseCeiling" => "an absent or blank value uses the default; the value is read with parseInt and anything that is not an integer of at least 1 falls back to the default",
        _ => "unclassified",
    }
}

fn val_json(v: Val, unit: &str) -> J {
    jo(vec![
        ("source", js(if v.env.is_some() { "env" } else { "literal" })),
        ("env", jopt(v.env.as_deref())),
        ("default", jn(usize::try_from(v.default).unwrap_or(0))),
        ("unit", js(unit)),
        ("parsing", js(v.parsing)),
        ("invalid_value", js(parsing_behaviour(v.parsing))),
        ("invalid_value_fallback", v.fallback.map_or(J::Null, |f| jn(usize::try_from(f).unwrap_or(0)))),
        ("declared_at", J::A(v.decls)),
    ])
}

const LOGIN_POLICY: &str = "src/modules/identity-access/application/login-policy.ts";
const MFA_CONFIG: &str = "src/lib/auth/mfa-config.ts";
const INVITATION_CONFIG: &str = "src/lib/auth/invitation-config.ts";

fn env_name_of(arg: &str) -> Option<String> {
    arg.trim().strip_prefix("process.env.").filter(|n| is_ident(n)).map(str::to_owned)
}

/// Decl facts for an env-backed const: the declaration line and the line that
/// reads `process.env.NAME`.
fn env_decl_facts(book: &Book, rel: &str, decl_line: usize, name: &str, env: &str) -> Res<Vec<J>> {
    let read = book.find_from(rel, &format!("process.env.{env}"), decl_line)?;
    if read > decl_line + 6 {
        return Err(format!("{rel}: process.env.{env} is not part of const {name}").into());
    }
    Ok(vec![book.fact(rel, decl_line, &format!("const {name}"))?, book.fact(rel, read, &format!("process.env.{env}"))?])
}

/// `policy.<field>` -> the `resolveLoginPolicyConfig` entry.
fn resolve_policy(book: &Book, field: &str) -> Res<Val> {
    let lines = book.lines(LOGIN_POLICY)?;
    let needle = format!("{field}: parsePositiveIntEnv(");
    let line = book.find(LOGIN_POLICY, &needle)?;
    let (group, _) = call_group(lines, line - 1, &format!("{field}: parsePositiveIntEnv"))?;
    let args = split_top(group_inner(&group), ',');
    if args.len() != 2 {
        return Err(format!("{LOGIN_POLICY}:{line}: parsePositiveIntEnv expects (name, default)").into());
    }
    let env = strip_quotes(&args[0]);
    let (_, default, const_fact) = const_num(book, LOGIN_POLICY, args[1].trim())?;
    let env_line = book.find_from(LOGIN_POLICY, &format!("\"{env}\""), line)?;
    Ok(Val {
        env: Some(env.clone()),
        default,
        parsing: "parsePositiveIntEnv",
        fallback: None,
        decls: vec![book.fact(LOGIN_POLICY, line, "parsePositiveIntEnv(")?, book.fact(LOGIN_POLICY, env_line, &format!("\"{env}\""))?, const_fact],
    })
}

/// `resolveMfa...()` -> `Number(env.X)` guarded in the function, default from a const.
fn resolve_env_fn(book: &Book, rel: &str, name: &str) -> Res<Val> {
    let lines = book.lines(rel)?;
    let start = book.find(rel, &format!("export function {name}("))?;
    let body: Vec<(usize, &String)> = lines.iter().enumerate().skip(start - 1).take(12).map(|(i, l)| (i + 1, l)).collect();
    let (env_line, env) = body
        .iter()
        .find_map(|(n, l)| {
            let at = l.find("Number(env.")?;
            let rest = &l[at + "Number(env.".len()..];
            let end = rest.find(')')?;
            Some((*n, rest[..end].to_owned()))
        })
        .ok_or_else(|| format!("{rel}: {name}: no Number(env.X) read"))?;
    let default_name = body
        .iter()
        .find_map(|(_, l)| {
            let at = l.find("DEFAULT_")?;
            let tail = &l[at..];
            let end = tail.find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')).unwrap_or(tail.len());
            Some(tail[..end].to_owned())
        })
        .ok_or_else(|| format!("{rel}: {name}: no DEFAULT_ constant"))?;
    let (_, default, const_fact) = const_num(book, rel, &default_name)?;
    Ok(Val {
        env: Some(env.clone()),
        default,
        parsing: "Number-env-guarded-in-function",
        fallback: None,
        decls: vec![book.fact(rel, start, &format!("export function {name}"))?, book.fact(rel, env_line, &format!("Number(env.{env})"))?, const_fact],
    })
}

/// `Number.isFinite(X) && X > 0 ? X : D` -> (X, D).
fn parse_guard(e: &str) -> Option<(String, i64)> {
    let rest = e.strip_prefix("Number.isFinite(")?;
    let close = rest.find(')')?;
    let ident = rest[..close].trim();
    if !is_ident(ident) {
        return None;
    }
    let tail = rest[close + 1..].trim();
    let tail = tail.strip_prefix("&&")?.trim();
    let tail = tail.strip_prefix(ident)?.trim();
    let tail = tail.strip_prefix("> 0")?.trim();
    let tail = tail.strip_prefix('?')?.trim();
    let tail = tail.strip_prefix(ident)?.trim();
    let tail = tail.strip_prefix(':')?.trim();
    Some((ident.to_owned(), lit_product(tail)?))
}

/// A numeric expression (a count, or seconds) -> how it is configured.
fn resolve_value(book: &Book, rel: &str, expr: &str) -> Res<Val> {
    let e = strip_outer_parens(&norm(expr));
    if let Some(v) = lit_product(&e) {
        return Ok(Val::literal(v));
    }
    if let Some((ident, fallback)) = parse_guard(&e) {
        let mut v = resolve_value(book, rel, &ident)?;
        if v.parsing != "Number-coercion-unguarded" && v.parsing != "parsePositiveIntSetting" {
            return Err(format!("{rel}: guard around `{ident}` which is neither a bare Number(env) coercion nor a parsed setting").into());
        }
        let guard = book.find(rel, &format!("Number.isFinite({ident})"))?;
        if v.parsing == "Number-coercion-unguarded" {
            v.parsing = "Number-coercion-guarded";
        }
        v.fallback = Some(fallback);
        v.decls.push(book.fact(rel, guard, &format!("Number.isFinite({ident})"))?);
        return Ok(v);
    }
    if let Some(field) = e.strip_prefix("policy.") {
        return resolve_policy(book, field);
    }
    if let Some(name) = e.strip_suffix("()")
        && name.starts_with("resolveMfa")
    {
        return resolve_env_fn(book, MFA_CONFIG, name);
    }
    if is_ident(&e) {
        let (line, init) = decl_init(book, rel, &e)?.ok_or_else(|| format!("{rel}: `{e}` is not declared in this file"))?;
        let init = strip_outer_parens(&init);
        if let Some(v) = lit_product(&init) {
            let mut val = Val::literal(v);
            val.decls.push(book.fact(rel, line, &format!("const {e}"))?);
            return Ok(val);
        }
        if let Some(rest) = init.strip_prefix("parsePositiveIntSetting(") {
            let args = split_top(rest.strip_suffix(')').unwrap_or(rest), ',');
            let env = args.first().and_then(|a| env_name_of(a)).ok_or_else(|| format!("{rel}: {e}: parsePositiveIntSetting without process.env.X"))?;
            let default = args.get(1).and_then(|a| lit_product(a)).ok_or_else(|| format!("{rel}: {e}: non-literal default"))?;
            return Ok(Val { env: Some(env.clone()), default, parsing: "parsePositiveIntSetting", fallback: None, decls: env_decl_facts(book, rel, line, &e, &env)? });
        }
        if let Some(rest) = init.strip_prefix("Number(") {
            let inner = rest.strip_suffix(')').unwrap_or(rest);
            if let Some((left, right)) = inner.split_once("??") {
                let env = env_name_of(left).ok_or_else(|| format!("{rel}: {e}: Number() without process.env.X"))?;
                let default = lit_product(right.trim()).ok_or_else(|| format!("{rel}: {e}: non-literal default"))?;
                return Ok(Val { env: Some(env.clone()), default, parsing: "Number-coercion-unguarded", fallback: None, decls: env_decl_facts(book, rel, line, &e, &env)? });
            }
        }
        if init.ends_with("()") {
            return resolve_value(book, rel, &init);
        }
        return Err(format!("{rel}:{line}: cannot classify the initialiser of `{e}`").into());
    }
    Err(format!("{rel}: cannot resolve numeric expression `{e}`").into())
}

/// A window expression in milliseconds -> seconds.
fn resolve_window(book: &Book, rel: &str, expr: &str) -> Res<Val> {
    let e = norm(expr);
    if let Some(inner) = e.strip_suffix("* 1000") {
        return resolve_value(book, rel, inner.trim());
    }
    let mut v = resolve_value(book, rel, &e)?;
    if v.default % 1000 != 0 {
        return Err(format!("{rel}: window `{e}` is not a whole number of seconds").into());
    }
    v.default /= 1000;
    Ok(v)
}

enum Config {
    Resolved(Box<Val>, Box<Val>),
    Param(String),
}

/// `resolveInvitationRateLimit()` -> (max, window seconds).
fn resolve_invitation(book: &Book) -> Res<(Val, Val)> {
    let rel = INVITATION_CONFIG;
    let start = book.find(rel, "export function resolveInvitationRateLimit(")?;
    let one = |var: &str, unit_guard: &str| -> Res<Val> {
        let needle = format!("const {var} = Number(env.");
        let line = book.find_from(rel, &needle, start)?;
        let text = &book.lines(rel)?[line - 1];
        let rest = &text[text.find("Number(env.").ok_or("invitation: Number(env.")? + "Number(env.".len()..];
        let (env, default) = rest.split_once("??").ok_or("invitation: ??")?;
        let env = env.trim().to_owned();
        let default = lit_product(default.trim().trim_end_matches(");").trim()).ok_or("invitation: default")?;
        let guard_line = book.find_from(rel, unit_guard, line)?;
        let guard_text = &book.lines(rel)?[guard_line - 1];
        let fallback_text = guard_text.split(':').next_back().ok_or("invitation: fallback")?;
        let digits: String = fallback_text.trim().chars().take_while(|c| c.is_ascii_digit() || *c == '_').collect();
        let fallback = lit_product(&digits).ok_or("invitation: fallback literal")?;
        Ok(Val {
            env: Some(env.clone()),
            default,
            parsing: "Number-coercion-guarded",
            fallback: Some(fallback),
            decls: vec![book.fact(rel, line, &format!("Number(env.{env}"))?, book.fact(rel, guard_line, unit_guard)?],
        })
    };
    let max = one("maxAttempts", "maxAttempts > 0")?;
    let window = one("windowSec", "windowSec > 0")?;
    Ok((max, window))
}

fn config_props(text: &str) -> Vec<(String, String)> {
    split_top(group_inner(text), ',')
        .into_iter()
        .filter(|p| !p.is_empty())
        .map(|p| split_pair(&p, ':').unwrap_or_else(|| (p.clone(), p.clone())))
        .collect()
}

/// Names of the parameters of the function that encloses `line0` (0-based).
fn enclosing_params(lines: &[String], line0: usize) -> Option<(String, Vec<String>)> {
    for index in (0..=line0.min(lines.len().saturating_sub(1))).rev() {
        let t = lines[index].trim_start();
        let rest = t.strip_prefix("export ").unwrap_or(t);
        let rest = rest.strip_prefix("async ").unwrap_or(rest);
        if let Some(after) = rest.strip_prefix("function ") {
            let name: String = after.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
            let col = lines[index].find(&format!("function {name}"))? + "function ".len() + name.len();
            let group = scan(lines, index, col, Stop::Group).ok()?;
            let params = split_top(group_inner(&group), ',')
                .into_iter()
                .map(|p| p.split([':', '=']).next().unwrap_or("").trim().to_owned())
                .collect();
            return Some((name, params));
        }
    }
    None
}

/// The config argument of a limiter call: an object literal, a const object, a
/// `resolveInvitationRateLimit()` result, or a parameter of the enclosing function.
fn resolve_config(book: &Book, rel: &str, call_line1: usize, span_end1: usize, expr: &str) -> Res<Config> {
    let e = norm(expr);
    let lines = book.lines(rel)?;
    if e.starts_with('{') {
        return resolve_object(book, rel, call_line1, span_end1, &e).map(|(m, w)| Config::Resolved(Box::new(m), Box::new(w)));
    }
    if e == "resolveInvitationRateLimit()" {
        let (m, w) = resolve_invitation(book)?;
        return Ok(Config::Resolved(Box::new(m), Box::new(w)));
    }
    if is_ident(&e) {
        if let Some((line, init)) = decl_init(book, rel, &e)? {
            let (max, window) = if init.starts_with('{') {
                resolve_object(book, rel, line, line + init.matches('\n').count(), &init)?
            } else if init == "resolveInvitationRateLimit()" {
                resolve_invitation(book)?
            } else {
                return Err(format!("{rel}:{line}: cannot classify rate-limit config `{e}`").into());
            };
            let mut max = max;
            max.decls.insert(0, book.fact(rel, line, &format!("const {e}"))?);
            return Ok(Config::Resolved(Box::new(max), Box::new(window)));
        }
        if let Some((_, params)) = enclosing_params(lines, call_line1 - 1)
            && params.contains(&e)
        {
            return Ok(Config::Param(e));
        }
    }
    Err(format!("{rel}:{call_line1}: cannot resolve rate-limit config `{e}`").into())
}

fn resolve_object(book: &Book, rel: &str, from_line1: usize, to_line1: usize, text: &str) -> Res<(Val, Val)> {
    let props = config_props(text);
    let get = |name: &str| props.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone()).ok_or_else(|| format!("{rel}:{from_line1}: config has no {name}"));
    let lines = book.lines(rel)?;
    let literal_fact = |prop: &str, value: &str| -> Res<J> {
        let anchor = format!("{prop}: {}", norm(value));
        let line = (from_line1..=to_line1.min(lines.len())).find(|n| lines[n - 1].contains(&anchor)).ok_or_else(|| format!("{rel}:{from_line1}: no line carries `{anchor}`"))?;
        book.fact(rel, line, &anchor)
    };
    let max_expr = get("maxAttempts")?;
    let window_expr = get("windowMs")?;
    let mut max = resolve_value(book, rel, &max_expr)?;
    if max.decls.is_empty() {
        max.decls.push(literal_fact("maxAttempts", &max_expr)?);
    }
    let mut window = resolve_window(book, rel, &window_expr)?;
    if window.decls.is_empty() {
        window.decls.push(literal_fact("windowMs", &window_expr)?);
    }
    Ok((max, window))
}

// ------------------------------------------------------------ rate limits ----

const RATE_LIMIT_MODULE: &str = "src/lib/security/rate-limit.ts";
const AUTH_RATE_LIMIT_MODULE: &str = "src/lib/security/auth-rate-limit.ts";
const ENV_THRESHOLDS: &str = "src/lib/security/env-thresholds.ts";
const TURNSTILE_MODULE: &str = "src/lib/security/turnstile.ts";
const LIMITER_CALLS: [&str; 3] = ["checkSharedRateLimit(", "checkAuthRateLimit(", "checkRateLimit("];

struct RateOutput {
    json: J,
    sites: usize,
    buckets: usize,
    classes: usize,
    used: Vec<String>,
}

fn ident_before(line: &str, at: usize) -> bool {
    line[..at].chars().next_back().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Start lines (0-based) and names of the exported HTTP handlers of a file.
fn handler_starts(lines: &[String]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        for prefix in ["export const ", "export async function ", "export function "] {
            if let Some(rest) = line.strip_prefix(prefix) {
                let name: String = rest.chars().take_while(char::is_ascii_alphabetic).collect();
                if HTTP_METHODS.contains(&name.as_str()) {
                    out.push((index, name));
                }
            }
        }
    }
    out
}

/// The HTTP handler (`GET`, `POST`, ...) that encloses `line0`. A call inside a
/// helper that precedes the handlers belongs to the single handler that calls it.
fn enclosing_handler(lines: &[String], line0: usize) -> Option<String> {
    let handlers = handler_starts(lines);
    if let Some((_, name)) = handlers.iter().rev().find(|(start, _)| *start <= line0) {
        return Some(name.clone());
    }
    let (helper, _) = enclosing_params(lines, line0)?;
    let needle = format!("{helper}(");
    let mut found = BTreeSet::new();
    for (position, (start, method)) in handlers.iter().enumerate() {
        let end = handlers.get(position + 1).map_or(lines.len(), |next| next.0);
        if lines[*start..end].iter().any(|l| is_code_line(l) && l.contains(&needle)) {
            found.insert(method.clone());
        }
    }
    if found.len() == 1 { found.into_iter().next() } else { None }
}

fn route_for(file: &str) -> Option<String> {
    file.strip_prefix("src/pages/").map(route_of)
}

struct Raw {
    file: String,
    line: usize,
    needle: &'static str,
}

/// Every non-comment call of a limiter function outside the two limiter modules.
fn limiter_calls(files: &Files) -> Vec<Raw> {
    let mut out = Vec::new();
    for (file, lines) in files {
        if !file.starts_with("src/") || file == RATE_LIMIT_MODULE || file == AUTH_RATE_LIMIT_MODULE {
            continue;
        }
        for (index, line) in lines.iter().enumerate() {
            if !is_code_line(line) {
                continue;
            }
            for needle in LIMITER_CALLS {
                if let Some(at) = line.find(needle)
                    && !ident_before(line, at)
                {
                    out.push(Raw { file: file.clone(), line: index + 1, needle });
                }
            }
        }
    }
    out
}

/// A dimension of a bucket key: `name`, the source expression, and where an
/// attacker-controlled value could come from.
fn dim(name: &str, expr: &str, origin: &[String]) -> J {
    jo(vec![("name", js(name)), ("expr", js(expr)), ("origin", jstrs(origin))])
}

fn classify_key_expr(expr: &str) -> Res<(&'static str, &'static str)> {
    match expr.trim() {
        "clientIp" | "resolveClientIp(request, clientAddress)" => Ok(("client_ip", "resolveClientIp(request, clientAddress)")),
        "begun.actorTenantUserId" => Ok(("principal", "authenticated tenant user id")),
        other => Err(format!("unclassified rate-limit key part `{other}`").into()),
    }
}

/// Where a tenant-identifying expression comes from, read off its declaration.
fn tenant_origin(book: &Book, rel: &str, expr: &str) -> Res<(Vec<String>, Vec<J>)> {
    let e = expr.trim();
    let mut origin = Vec::new();
    let mut facts = Vec::new();
    let mut inspect = |name: &str| -> Res<()> {
        let lines = book.lines(rel)?;
        let mut hit = None;
        for (index, line) in lines.iter().enumerate() {
            let t = line.trim_start();
            if (t.starts_with(&format!("const {name} =")) || t.starts_with(&format!("const {{ {name}")) || t.starts_with(&format!("const {{ tenantId: {name}"))) && hit.is_none() {
                hit = Some(index);
            }
        }
        if let Some(index) = hit {
            let col = lines[index].find('=').ok_or("tenant decl: =")? + 1;
            let text = scan(lines, index, col, Stop::Semicolon)?;
            let t = text.replace(char::is_whitespace, "");
            if t.contains("\"x-awcms-tenant-id\"") {
                origin.push("request header x-awcms-tenant-id".to_owned());
            }
            if t.contains("TENANT_COOKIE_NAME") {
                origin.push("tenant cookie".to_owned());
            }
            if t.contains("searchParams.get(\"tenantId\")") {
                origin.push("query parameter tenantId".to_owned());
            }
            if t.contains("resolveAuthInputs(") {
                origin.push("resolveAuthInputs(request, cookies)".to_owned());
            }
            if t.contains("body.tenantId") {
                origin.push("request body tenantId".to_owned());
            }
            facts.push(book.fact(rel, index + 1, &format!("{name}"))?);
        }
        Ok(())
    };
    match e {
        "TENANTLESS_RATE_LIMIT_KEY" => {
            let (value, fact) = const_str(book, rel, e)?;
            origin.push(format!("constant \"{value}\""));
            facts.push(fact);
        }
        "body.tenantId" | "body.targetTenantId" => origin.push(format!("request body {}", e.trim_start_matches("body."))),
        "inputs.tenantId" => {
            inspect("inputs")?;
            if origin.is_empty() {
                origin.push("local variable `inputs`".to_owned());
            }
        }
        _ if is_ident(e) => inspect(e)?,
        _ => {}
    }
    if origin.is_empty() {
        origin.push("unclassified".to_owned());
    }
    Ok((origin, facts))
}

/// Template literal -> (prefix before the first `${`, the `${...}` expressions).
fn template_parts(arg: &str) -> Res<(String, Vec<String>)> {
    let t = arg.trim();
    let inner = t.strip_prefix('`').and_then(|x| x.strip_suffix('`')).ok_or_else(|| format!("rate-limit key is not a template literal: {t}"))?;
    let prefix_end = inner.find("${").unwrap_or(inner.len());
    let mut exprs = Vec::new();
    let mut rest = &inner[prefix_end..];
    while let Some(start) = rest.find("${") {
        let after = &rest[start + 2..];
        let end = after.find('}').ok_or("unterminated ${ in key")?;
        exprs.push(after[..end].to_owned());
        rest = &after[end + 1..];
    }
    Ok((inner[..prefix_end].to_owned(), exprs))
}

struct Site {
    file: String,
    line: usize,
    json: J,
    route: Option<String>,
    method: Option<String>,
    bucket_keys: Vec<(String, String)>,
    dims: Vec<String>,
    kinds: Vec<&'static str>,
}

fn auth_source_bucket(book: &Book) -> Res<(J, Vec<J>)> {
    let rel = AUTH_RATE_LIMIT_MODULE;
    let lines = book.lines(rel)?;
    let first = book.find(rel, "const source = await checkSharedRateLimit(")?;
    let (group, span) = call_group(lines, first - 1, "checkSharedRateLimit")?;
    let args = split_top(group_inner(&group), ',');
    let (prefix, exprs) = template_parts(&args[0])?;
    let prefix_name = exprs.first().ok_or("auth source key")?;
    let _ = prefix_name;
    let max_text = config_props(&args[1]).into_iter().find(|(k, _)| k == "maxAttempts").ok_or("auth source max")?.1;
    let win_text = config_props(&args[1]).into_iter().find(|(k, _)| k == "windowMs").ok_or("auth source window")?.1;
    let ceiling = |text: &str, seconds: bool| -> Res<Val> {
        let inner = text.trim().trim_end_matches("* 1000").trim();
        let inner = inner.strip_prefix("parseCeiling(").and_then(|x| x.strip_suffix(')')).ok_or("parseCeiling(")?;
        let parts = split_top(inner, ',');
        let env = env_name_of(&parts[0]).ok_or("parseCeiling env")?;
        let (_, default, const_fact) = const_num(book, rel, parts[1].trim())?;
        let _ = seconds;
        let env_line = book.find_from(rel, &format!("process.env.{env}"), first)?;
        Ok(Val {
            env: Some(env.clone()),
            default,
            parsing: "parseCeiling",
            fallback: Some(default),
            decls: vec![book.fact(rel, env_line, &format!("process.env.{env}"))?, const_fact],
        })
    };
    let max = ceiling(&max_text, false)?;
    let window = ceiling(&win_text, true)?;
    let key_const = const_str(book, rel, "SOURCE_BUCKET_PREFIX")?;
    let _ = (prefix, span);
    let bucket = jo(vec![
        ("name", js("source")),
        ("key_template", js(&format!("{}:${{clientIp}}", key_const.0))),
        ("shared_across_routes", J::B(true)),
        ("checked_first", J::B(true)),
        ("dimensions", J::A(vec![dim("client_ip", "input.clientIp", &[])])),
        ("max", val_json(max, "requests")),
        ("window", val_json(window, "seconds")),
        ("source", book.fact(rel, first, "checkSharedRateLimit(")?),
    ]);
    Ok((bucket, vec![key_const.1]))
}

fn build_site(book: &Book, raw: &Raw, via: Option<(&str, usize)>) -> Res<Vec<Site>> {
    let rel = raw.file.as_str();
    let lines = book.lines(rel)?;
    let (group, span) = call_group(lines, raw.line - 1, raw.needle.trim_end_matches('('))?;
    let span_end = raw.line + span;
    let args = split_top(group_inner(&group), ',');
    let route = route_for(rel);
    let method = if route.is_some() { Some(enclosing_handler(lines, raw.line - 1).ok_or_else(|| format!("{rel}:{}: no HTTP handler encloses the limiter call", raw.line))?) } else { None };
    let surface = if route.is_some() { "route" } else { "module" };
    let limiter = raw.needle.trim_end_matches('(');
    let call_fact = book.fact(rel, raw.line, raw.needle)?;
    let mut buckets: Vec<J> = Vec::new();
    let mut bucket_keys: Vec<(String, String)> = Vec::new();
    let mut dim_names: Vec<String> = Vec::new();
    let mut kinds: Vec<&'static str> = Vec::new();
    let mut extra_facts: Vec<J> = Vec::new();

    let finish = |buckets: Vec<J>, bucket_keys: Vec<(String, String)>, dim_names: Vec<String>, kinds: Vec<&'static str>, extra: Vec<J>| -> Site {
        let json = jo(vec![
            ("file", js(rel)),
            ("line", jn(raw.line)),
            ("limiter", js(limiter)),
            ("surface", js(surface)),
            ("route", jopt(route.as_deref())),
            ("method", jopt(method.as_deref())),
            ("call", call_fact_clone(&call_fact)),
            ("invoked_through", via.map_or(J::Null, |(f, l)| jo(vec![("file", js(f)), ("line", jn(l))]))),
            ("buckets", J::A(buckets)),
            ("supporting_facts", J::A(extra)),
        ]);
        Site { file: rel.to_owned(), line: raw.line, json, route: route.clone(), method: method.clone(), bucket_keys, dims: dim_names, kinds }
    };

    if raw.needle == "checkAuthRateLimit(" {
        let object = args.first().ok_or("checkAuthRateLimit without argument")?;
        let props = config_props(object);
        let prop = |name: &str| props.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone());
        let client_expr = prop("clientIp").ok_or_else(|| format!("{rel}:{}: clientIp missing", raw.line))?;
        let tenant_expr = prop("tenantId").ok_or_else(|| format!("{rel}:{}: tenantId missing", raw.line))?;
        let scope_expr = prop("scope").ok_or_else(|| format!("{rel}:{}: scope missing", raw.line))?;
        let scope = if scope_expr.starts_with('"') { strip_quotes(&scope_expr) } else { let (v, f) = const_str(book, rel, &scope_expr)?; extra_facts.push(f); v };
        let config_expr = prop("config").ok_or_else(|| format!("{rel}:{}: config missing", raw.line))?;
        let (max, window) = match resolve_config(book, rel, raw.line, span_end, &config_expr)? {
            Config::Resolved(m, w) => (*m, *w),
            Config::Param(p) => return Err(format!("{rel}:{}: config `{p}` is a parameter of an auth limiter caller", raw.line).into()),
        };
        let (client_name, client_origin) = classify_key_expr(&client_expr)?;
        let (origin, tenant_facts) = tenant_origin(book, rel, &tenant_expr)?;
        extra_facts.extend(tenant_facts);
        let (source_bucket, source_facts) = auth_source_bucket(book)?;
        extra_facts.extend(source_facts);
        let scope_key = format!("${{clientIp}}:${{tenantId}}:{scope}");
        buckets.push(source_bucket);
        bucket_keys.push(("source".to_owned(), "auth-source:${clientIp}".to_owned()));
        buckets.push(jo(vec![
            ("name", js("scope")),
            ("key_template", js(&scope_key)),
            ("shared_across_routes", J::B(false)),
            ("checked_first", J::B(false)),
            ("dimensions", J::A(vec![dim(client_name, &client_expr, &[client_origin.to_owned()]), dim("tenant", &tenant_expr, &origin), dim("scope", &format!("\"{scope}\""), &["literal in source".to_owned()])])),
            ("max", val_json(max, "requests")),
            ("window", val_json(window, "seconds")),
            ("source", book.fact(rel, raw.line, raw.needle)?),
        ]));
        bucket_keys.push(("scope".to_owned(), scope_key));
        dim_names.extend(["client_ip", "tenant", "scope"].map(str::to_owned));
        kinds.push("tenant-keyed");
        return Ok(vec![finish(buckets, bucket_keys, dim_names, kinds, extra_facts)]);
    }

    if raw.needle != "checkSharedRateLimit(" {
        return Err(format!("{rel}:{}: unexpected direct call of {limiter}", raw.line).into());
    }
    let key_arg = args.first().ok_or("limiter call without key")?;
    let config_arg = args.get(1).ok_or("limiter call without config")?;
    let (prefix, exprs) = template_parts(key_arg)?;
    match resolve_config(book, rel, raw.line, span_end, config_arg)? {
        Config::Resolved(max, window) => {
            let mut dims = Vec::new();
            for e in &exprs {
                let (name, origin) = classify_key_expr(e)?;
                dims.push(dim(name, e, &[origin.to_owned()]));
                dim_names.push(name.to_owned());
            }
            let key_template = format!("{prefix}{}", exprs.iter().map(|e| format!("${{{}}}", classify_key_expr(e).map_or("?", |c| c.0))).collect::<Vec<_>>().join(":"));
            buckets.push(jo(vec![
                ("name", js("route")),
                ("key_template", js(&key_template)),
                ("shared_across_routes", J::B(false)),
                ("checked_first", J::B(true)),
                ("dimensions", J::A(dims)),
                ("max", val_json(*max, "requests")),
                ("window", val_json(*window, "seconds")),
                ("source", book.fact(rel, raw.line, raw.needle)?),
            ]));
            bucket_keys.push(("route".to_owned(), key_template));
            Ok(vec![finish(buckets, bucket_keys, dim_names, kinds, extra_facts)])
        }
        Config::Param(config_param) => {
            let (fname, params) = enclosing_params(lines, raw.line - 1).ok_or("wrapper: no enclosing function")?;
            let key_param = exprs.iter().find(|e| params.contains(e)).cloned().ok_or_else(|| format!("{rel}:{}: key does not use a wrapper parameter", raw.line))?;
            let config_index = params.iter().position(|p| *p == config_param).ok_or("wrapper config index")?;
            let key_index = params.iter().position(|p| *p == key_param).ok_or("wrapper key index")?;
            let mut out = Vec::new();
            for (caller_file, caller_lines) in book.files {
                if !caller_file.starts_with("src/") || caller_file == rel {
                    continue;
                }
                for (index, line) in caller_lines.iter().enumerate() {
                    if !is_code_line(line) {
                        continue;
                    }
                    let needle = format!("{fname}(");
                    let Some(at) = line.find(&needle) else { continue };
                    if ident_before(line, at) || line.contains("import ") {
                        continue;
                    }
                    let (cgroup, cspan) = call_group(caller_lines, index, &fname)?;
                    let cargs = split_top(group_inner(&cgroup), ',');
                    let key_literal = strip_quotes(cargs.get(key_index).ok_or("wrapper caller key arg")?);
                    let cfg_arg = cargs.get(config_index).ok_or("wrapper caller config arg")?;
                    let caller_route = route_for(caller_file);
                    let caller_method = if caller_route.is_some() { Some(enclosing_handler(caller_lines, index).ok_or_else(|| format!("{caller_file}:{}: no handler", index + 1))?) } else { None };
                    let (max, window) = match resolve_config(book, caller_file, index + 1, index + 1 + cspan, cfg_arg)? {
                        Config::Resolved(m, w) => (*m, *w),
                        Config::Param(p) => return Err(format!("{caller_file}:{}: wrapper config `{p}` unresolved", index + 1).into()),
                    };
                    let dims_json = vec![dim("client_ip", "resolveClientIp(request, clientAddress)", &["resolveClientIp(request, clientAddress)".to_owned()])];
                    let key_template = format!("{key_literal}:${{client_ip}}");
                    let b = jo(vec![
                        ("name", js("route")),
                        ("key_template", js(&key_template)),
                        ("shared_across_routes", J::B(false)),
                        ("checked_first", J::B(true)),
                        ("dimensions", J::A(dims_json)),
                        ("max", val_json(max, "requests")),
                        ("window", val_json(window, "seconds")),
                        ("source", book.fact(rel, raw.line, raw.needle)?),
                    ]);
                    let json = jo(vec![
                        ("file", js(caller_file)),
                        ("line", jn(index + 1)),
                        ("limiter", js(limiter)),
                        ("surface", js(if caller_route.is_some() { "route" } else { "module" })),
                        ("route", jopt(caller_route.as_deref())),
                        ("method", jopt(caller_method.as_deref())),
                        ("call", book.fact(caller_file, index + 1, &needle)?),
                        ("invoked_through", jo(vec![("file", js(rel)), ("line", jn(raw.line)), ("function", js(&fname))])),
                        ("buckets", J::A(vec![b])),
                        ("supporting_facts", J::A(vec![book.fact(rel, raw.line, raw.needle)?])),
                    ]);
                    out.push(Site {
                        file: caller_file.clone(),
                        line: index + 1,
                        json,
                        route: caller_route,
                        method: caller_method,
                        bucket_keys: vec![("route".to_owned(), key_template)],
                        dims: vec!["client_ip".to_owned()],
                        kinds: vec!["via-wrapper"],
                    });
                }
            }
            if out.is_empty() {
                return Err(format!("{rel}:{}: wrapper {fname} has no caller", raw.line).into());
            }
            Ok(out)
        }
    }
}

fn call_fact_clone(fact: &J) -> J {
    match fact {
        J::O(items) => J::O(items.iter().map(|(k, v)| (k.clone(), call_fact_clone(v))).collect()),
        J::A(items) => J::A(items.iter().map(call_fact_clone).collect()),
        J::S(s) => J::S(s.clone()),
        J::N(n) => J::N(*n),
        J::B(b) => J::B(*b),
        J::Null => J::Null,
    }
}

fn statement(id: &str, text: &str, value: Option<J>, facts: Vec<J>) -> J {
    jo(vec![("id", js(id)), ("statement", js(text)), ("value", value.unwrap_or(J::Null)), ("facts", J::A(facts))])
}

/// What the limiter module itself does, each statement tied to a cited line.
fn limiter_statements(book: &Book) -> Res<Vec<J>> {
    let rl = RATE_LIMIT_MODULE;
    let (_, cap, cap_fact) = const_num(book, rl, "BUCKET_HARD_CAP")?;
    let (_, sweep, sweep_fact) = const_num(book, rl, "BUCKET_SWEEP_INTERVAL_MS")?;
    let (_, timeout, timeout_fact) = const_num(book, rl, "RATE_LIMIT_COMMAND_TIMEOUT_MS")?;
    let (_, hops, hops_fact) = const_num(book, rl, "DEFAULT_TRUSTED_PROXY_HOP_COUNT")?;
    let n = |v: i64| Some(jn(usize::try_from(v).unwrap_or(0)));
    Ok(vec![
        statement(
            "algorithm",
            "fixed-window counter: the first hit opens a window, later hits in it increment, and the request is refused once the count exceeds maxAttempts",
            None,
            vec![book.fact_find(rl, "now - existing.windowStart >= config.windowMs")?, book.fact_find(rl, "existing.count > config.maxAttempts")?],
        ),
        statement("in-process-backend", "checkRateLimit counts in a per-process Map keyed by the bucket key", None, vec![book.fact_find(rl, "const buckets = new Map<string, Bucket>();")?]),
        statement(
            "redis-backend",
            "checkSharedRateLimit counts in Redis (INCR, PEXPIRE on first use) when a Redis client is configured; the window index is part of the Redis key",
            None,
            vec![
                book.fact_find(rl, "client.send(\"INCR\", [redisKey])")?,
                book.fact_find(rl, "client.send(\"PEXPIRE\", [")?,
                book.fact_find(rl, "const window = Math.floor(now / config.windowMs);")?,
                book.fact_find(rl, "namespace: \"rate-limit\"")?,
            ],
        ),
        statement("no-redis-fallback", "without Redis, checkSharedRateLimit delegates to the in-process checkRateLimit", None, vec![book.fact_find(rl, "return checkRateLimit(key, config, now);")?]),
        statement(
            "redis-failure-is-fail-open",
            "when Redis is configured but a command fails, the shared limiter allows the request",
            None,
            vec![book.fact_find(rl, "// Fail OPEN.")?, {
                let marker = book.find(rl, "// Fail OPEN.")?;
                book.fact_find_from(rl, "return { allowed: true };", marker)?
            }],
        ),
        statement("redis-command-timeout-ms", "per-command Redis timeout", n(timeout), vec![timeout_fact]),
        statement("in-process-bucket-cap", "hard cap on live in-process buckets; soonest-to-expire buckets are evicted beyond it", n(cap), vec![cap_fact]),
        statement("in-process-sweep-interval-ms", "interval between sweeps of elapsed buckets", n(sweep), vec![sweep_fact]),
        statement(
            "retry-after",
            "a refusal carries retryAfterSec = ceil(remaining window / 1000), minimum 1",
            None,
            vec![book.fact_find(rl, "retryAfterSec: Math.max(1, Math.ceil(remainingMs / 1000))")?],
        ),
        statement(
            "client-ip-trust",
            "X-Forwarded-For is honoured only when TRUSTED_PROXY_ENABLED is exactly \"true\"; otherwise the socket address is the client",
            None,
            vec![book.fact_find(rl, "process.env.TRUSTED_PROXY_ENABLED === \"true\"")?, book.fact_find(rl, "return clientAddress ?? \"unknown\";")?],
        ),
        statement(
            "client-ip-hop-count",
            "with trust enabled the client is the X-Forwarded-For entry TRUSTED_PROXY_HOP_COUNT positions from the right (default shown)",
            n(hops),
            vec![book.fact_find(rl, "process.env.TRUSTED_PROXY_HOP_COUNT")?, hops_fact, book.fact_find(rl, "parts[parts.length - hopCount]")?],
        ),
        statement(
            "threshold-parser-parsePositiveIntSetting",
            "parsePositiveIntSetting rejects non-finite, non-integer and non-positive values and returns the default",
            None,
            vec![book.fact_find(ENV_THRESHOLDS, "export function parsePositiveIntSetting(")?, book.fact_find(ENV_THRESHOLDS, "!Number.isFinite(parsed) || !Number.isInteger(parsed) || parsed <= 0")?],
        ),
    ])
}

struct Control {
    id: &'static str,
    json: J,
    applies_to: Vec<(String, String)>,
}

fn applies_to_json(applies: &[(String, String)]) -> J {
    J::A(applies.iter().map(|(r, m)| jo(vec![("route", js(r)), ("method", js(m))])).collect())
}

/// Route and method that enclose a (file, 1-based line).
fn route_method(book: &Book, file: &str, line: usize) -> Res<(String, String)> {
    let lines = book.lines(file)?;
    let route = route_for(file).ok_or_else(|| format!("{file}: not a route file"))?;
    let method = enclosing_handler(lines, line - 1).ok_or_else(|| format!("{file}:{line}: no handler"))?;
    Ok((route, method))
}

fn code_lines_with(book: &Book, file: &str, needle: &str) -> Res<Vec<usize>> {
    let lines = book.lines(file)?;
    Ok(lines.iter().enumerate().filter(|(_, l)| is_code_line(l) && l.contains(needle) && !l.contains("import ")).map(|(i, _)| i + 1).collect())
}

fn other_controls(book: &Book, sites: &[Site]) -> Res<(Vec<Control>, J)> {
    let mut controls = Vec::new();

    // Password lockout: per principal, counted in the database.
    let login = "src/pages/api/v1/auth/login.ts";
    let store = "src/modules/identity-access/application/principal-store.ts";
    let max_attempts = resolve_policy(book, "maxFailedAttempts")?;
    let lockout_line = book.find(LOGIN_POLICY, "lockoutMinutes: DEFAULT_LOCKOUT_MINUTES")?;
    let (_, lockout_minutes, lockout_const) = const_num(book, LOGIN_POLICY, "DEFAULT_LOCKOUT_MINUTES")?;
    let mut lockout = Val::literal(lockout_minutes);
    lockout.decls = vec![book.fact(LOGIN_POLICY, lockout_line, "lockoutMinutes: DEFAULT_LOCKOUT_MINUTES")?, lockout_const];
    let mut recorded = Vec::new();
    let mut applies = BTreeSet::new();
    for line in code_lines_with(book, login, "recordPrincipalLoginFailure(")? {
        recorded.push(book.fact(login, line, "recordPrincipalLoginFailure(")?);
        applies.insert(route_method(book, login, line)?);
    }
    if recorded.is_empty() {
        return Err("login.ts no longer calls recordPrincipalLoginFailure".into());
    }
    let update = book.find(store, "UPDATE awcms_principals")?;
    controls.push(Control {
        id: "password-lockout",
        applies_to: applies.into_iter().collect(),
        json: jo(vec![
            ("id", js("password-lockout")),
            ("kind", js("per-account lockout")),
            ("dimensions", J::A(vec![dim("principal", "awcms_principals.id", &[])])),
            ("independent_of_client_ip", J::B(true)),
            ("max_failed_attempts", val_json(max_attempts, "attempts")),
            ("lockout_duration", val_json(lockout, "minutes")),
            ("enforced_by", J::A(vec![book.fact(store, update, "UPDATE awcms_principals")?, book.fact_find(store, "failed_login_count = failed_login_count + 1")?, book.fact_find(store, "CASE WHEN failed_login_count + 1 >=")?])),
            ("counted_at", J::A(recorded)),
        ]),
    });

    // MFA factor lockout.
    let mfa = "src/modules/identity-access/application/mfa.ts";
    let max_verify = resolve_env_fn(book, MFA_CONFIG, "resolveMfaMaxVerifyAttempts")?;
    let mfa_lockout = resolve_env_fn(book, MFA_CONFIG, "resolveMfaLockoutMinutes")?;
    controls.push(Control {
        id: "mfa-factor-lockout",
        applies_to: Vec::new(),
        json: jo(vec![
            ("id", js("mfa-factor-lockout")),
            ("kind", js("per-factor lockout")),
            ("dimensions", J::A(vec![dim("mfa_factor", "awcms_identity_mfa_factors row", &[])])),
            ("independent_of_client_ip", J::B(true)),
            ("max_failed_attempts", val_json(max_verify, "attempts")),
            ("lockout_duration", val_json(mfa_lockout, "minutes")),
            (
                "enforced_by",
                J::A(vec![
                    book.fact_find(mfa, "async function verifyFactorWithLockout(")?,
                    book.fact_find(mfa, "Independent of source IP")?,
                    book.fact_find(mfa, "resolveMfaMaxVerifyAttempts(env)")?,
                    book.fact_find(mfa, "resolveMfaLockoutMinutes(env)")?,
                ]),
            ),
        ]),
    });

    // Newsletter recipient cooldown.
    let subscribe_file = "src/pages/api/v1/newsletter/subscribe.ts";
    let directory = "src/modules/newsletter/application/subscriber-directory.ts";
    let cooldown = resolve_value(book, subscribe_file, "CONFIRMATION_COOLDOWN_SEC")?;
    let decl_line = book.find(subscribe_file, "const CONFIRMATION_COOLDOWN_SEC")?;
    let passed = book.fact_find_from(subscribe_file, "CONFIRMATION_COOLDOWN_SEC", decl_line + 6)?;
    let conflict = book.find(directory, "ON CONFLICT (tenant_id, email_normalized) DO UPDATE")?;
    let predicate = book.fact_find_from(directory, "confirmation_sent_at IS NULL", conflict)?;
    let interval = book.fact_find_from(directory, "make_interval(secs => ${confirmationCooldownSeconds})", conflict)?;
    let subscribe_route = route_method(book, subscribe_file, decl_line.max(book.find(subscribe_file, "export const POST")?))?;
    controls.push(Control {
        id: "newsletter-recipient-cooldown",
        applies_to: vec![subscribe_route],
        json: jo(vec![
            ("id", js("newsletter-recipient-cooldown")),
            ("kind", js("per-recipient cooldown on re-sending a confirmation e-mail")),
            ("dimensions", J::A(vec![dim("recipient", "(tenant_id, email_normalized)", &[]), dim("tenant", "tenant_id of the resolved tenant", &[])])),
            ("independent_of_client_ip", J::B(true)),
            ("cooldown", val_json(cooldown, "seconds")),
            ("passed_to_subscribe", passed),
            (
                "enforced_by",
                J::A(vec![
                    book.fact(directory, conflict, "ON CONFLICT (tenant_id, email_normalized) DO UPDATE")?,
                    predicate,
                    interval,
                    book.fact_find(directory, "confirmationCooldownSeconds: number")?,
                ]),
            ),
            ("keyed_on_unique_index", book.fact_find("sql/139_awcms_newsletter_schema.sql", "ON awcms_newsletter_subscribers (tenant_id, email_normalized);")?),
            ("response_is_neutral", book.fact_find(subscribe_file, "const NEUTRAL_MESSAGE")?),
        ]),
    });

    // Turnstile challenge: endpoints that call the enforcement helper.
    let mut turnstile_sites = Vec::new();
    let mut turnstile_applies = BTreeSet::new();
    for (file, lines) in book.files {
        if !file.starts_with("src/pages/") {
            continue;
        }
        for (index, line) in lines.iter().enumerate() {
            if is_code_line(line) && line.contains("enforceTurnstileIfRequired(") && !line.contains("import") && !line.trim_start().starts_with("enforceTurnstileIfRequired,") {
                let (route, method) = route_method(book, file, index + 1)?;
                let (group, _) = call_group(lines, index, "enforceTurnstileIfRequired")?;
                let args = split_top(group_inner(&group), ',');
                let options = args.get(2).map_or_else(String::new, |a| norm(a));
                turnstile_applies.insert((route.clone(), method.clone()));
                turnstile_sites.push(jo(vec![("route", js(&route)), ("method", js(&method)), ("options", js(&options)), ("call", book.fact(file, index + 1, "enforceTurnstileIfRequired(")?)]));
            }
        }
    }
    if turnstile_sites.is_empty() {
        return Err("no enforceTurnstileIfRequired call found".into());
    }
    controls.push(Control {
        id: "turnstile-challenge",
        applies_to: turnstile_applies.into_iter().collect(),
        json: jo(vec![
            ("id", js("turnstile-challenge")),
            ("kind", js("bot challenge verified server-side; active only when full-online security and Turnstile are both enabled")),
            ("dimensions", J::A(Vec::new())),
            ("independent_of_client_ip", J::B(false)),
            (
                "enforced_by",
                J::A(vec![
                    book.fact_find(TURNSTILE_MODULE, "export async function enforceTurnstileIfRequired(")?,
                    book.fact_find(TURNSTILE_MODULE, "isFullOnlineSecurityActive(env) && isTurnstileEnabled(env)")?,
                    book.fact_find(TURNSTILE_MODULE, "if (!isTurnstileRequired(env)) {")?,
                ]),
            ),
            ("call_sites", J::A(turnstile_sites)),
        ]),
    });

    // C-07: what the code does about recipient-oriented abuse control.
    let subscribe_site = sites
        .iter()
        .find(|s| s.route.as_deref() == Some("/api/v1/newsletter/subscribe") && s.method.as_deref() == Some("POST"))
        .ok_or("no limiter call on POST /api/v1/newsletter/subscribe")?;
    let sites_with_recipient = sites.iter().filter(|s| s.dims.iter().any(|d| d == "recipient")).count();
    let ip_only = subscribe_site.dims == ["client_ip"];
    let verdict = if ip_only && sites_with_recipient == 0 { "confirmed" } else { "not-confirmed" };
    let forgot = sites
        .iter()
        .find(|s| s.route.as_deref() == Some("/api/v1/auth/password/forgot"))
        .ok_or("no limiter call on POST /api/v1/auth/password/forgot")?;
    let reset_module = "src/modules/identity-access/application/password-reset.ts";
    let words = ["cooldown", "throttle", "resend", "recently"];
    let mut hits = Vec::new();
    for (index, line) in book.lines(reset_module)?.iter().enumerate() {
        if is_code_line(line) {
            for word in words {
                if line.to_lowercase().contains(word) {
                    hits.push(book.fact(reset_module, index + 1, word)?);
                }
            }
        }
    }
    let reset_function = book.fact_find(reset_module, "export async function requestPasswordReset(")?;
    let claim = jo(vec![
        ("id", js("C-07")),
        ("claim", js("per-IP rate limits alone could not prevent repeated email delivery to a victim address; recipient-oriented cooldown semantics were required")),
        ("verdict", js(verdict)),
        (
            "finding",
            js("POST /api/v1/newsletter/subscribe is limited per client IP only, and a separate per-recipient cooldown is enforced inside the upsert statement: a confirmation e-mail is re-issued to an existing (tenant_id, email_normalized) row only when its confirmation_sent_at is NULL or older than the cooldown. The cooldown is a predicate of the ON CONFLICT ... DO UPDATE branch, so a refused re-send returns the same neutral response as every other branch."),
        ),
        ("subscribe_limiter_dimensions", jstrs(&subscribe_site.dims)),
        ("subscribe_limiter_site", jo(vec![("file", js(&subscribe_site.file)), ("line", jn(subscribe_site.line))])),
        ("limiter_sites_with_a_recipient_dimension", jn(sites_with_recipient)),
        ("recipient_control", js("newsletter-recipient-cooldown")),
        (
            "comparison_password_forgot",
            jo(vec![
                ("limiter_dimensions", jstrs(&forgot.dims)),
                ("recipient_dimension_in_limiter", J::B(forgot.dims.iter().any(|d| d == "recipient"))),
                ("searched_module_function", reset_function),
                ("cooldown_words_searched", jstrs(&words)),
                ("cooldown_word_hits_in_password_reset_module", J::A(hits)),
                ("note", js("the forgot-password limiter key holds no identifier or address; whether any other control exists is not derivable from the limiter and is reported only as the word search above")),
            ]),
        ),
    ]);
    Ok((controls, claim))
}

fn build_rate_limits(book: &Book) -> Res<RateOutput> {
    let raws = limiter_calls(book.files);
    let mut sites: Vec<Site> = Vec::new();
    for raw in &raws {
        sites.extend(build_site(book, raw, None)?);
    }
    sites.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));

    // Sites sharing one bucket key share one counter.
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut ids: Vec<String> = Vec::new();
    for (index, site) in sites.iter().enumerate() {
        let id = format!("RL-{:03}", index + 1);
        for (name, key) in &site.bucket_keys {
            groups.entry(key.clone()).or_default().push(format!("{id}:{name}"));
        }
        ids.push(id);
    }
    let shared: Vec<J> = groups
        .iter()
        .filter(|(_, members)| members.len() > 1)
        .map(|(key, members)| jo(vec![("key_template", js(key)), ("buckets", jstrs(members))]))
        .collect();

    let (controls, claim) = other_controls(book, &sites)?;

    // Endpoint classes: one per (route, method), or per file for non-route modules.
    let mut classes: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
    for (index, site) in sites.iter().enumerate() {
        let key = match (&site.route, &site.method) {
            (Some(r), Some(m)) => (r.clone(), m.clone()),
            _ => (format!("module:{}", site.file), "-".to_owned()),
        };
        classes.entry(key).or_default().push(index);
    }
    let class_json: Vec<J> = classes
        .iter()
        .map(|((route, method), members)| {
            let linked: Vec<&str> = controls.iter().filter(|c| c.applies_to.iter().any(|(r, m)| r == route && m == method)).map(|c| c.id).collect();
            let is_module = route.starts_with("module:");
            jo(vec![
                ("route", if is_module { J::Null } else { js(route) }),
                ("method", if is_module { J::Null } else { js(method) }),
                ("module", if is_module { js(route.trim_start_matches("module:")) } else { J::Null }),
                ("rate_limit_sites", J::A(members.iter().map(|i| js(&ids[*i])).collect())),
                ("other_controls", jstrs(&linked)),
            ])
        })
        .collect();

    let mut parsing: BTreeMap<String, usize> = BTreeMap::new();
    let mut unguarded = Vec::new();
    for (index, site) in sites.iter().enumerate() {
        let mut text = String::new();
        render_json(&site.json, 0, &mut text);
        for kind in ["literal", "parsePositiveIntSetting", "parsePositiveIntEnv", "Number-coercion-guarded", "Number-coercion-unguarded", "Number-env-guarded-in-function"] {
            if text.contains(&format!("\"parsing\": \"{kind}\"")) {
                *parsing.entry(kind.to_owned()).or_default() += 1;
                if kind == "Number-coercion-unguarded" {
                    unguarded.push(js(&format!("{} {} {}", ids[index], site.method.as_deref().unwrap_or("-"), site.route.as_deref().unwrap_or(&site.file))));
                }
            }
        }
    }
    let bucket_total: usize = sites.iter().map(|s| s.bucket_keys.len()).sum();
    let routes_limited: BTreeSet<(String, String)> = sites.iter().filter_map(|s| Some((s.route.clone()?, s.method.clone()?))).collect();
    let tenant_keyed = sites.iter().filter(|s| s.kinds.contains(&"tenant-keyed")).count();
    let summary = jo(vec![
        ("limiter_call_lines", jn(raws.len())),
        ("call_sites", jn(sites.len())),
        ("buckets", jn(bucket_total)),
        ("distinct_route_method_pairs", jn(routes_limited.len())),
        ("endpoint_classes", jn(class_json.len())),
        ("tenant_keyed_sites", jn(tenant_keyed)),
        ("sites_with_recipient_dimension", jn(sites.iter().filter(|s| s.dims.iter().any(|d| d == "recipient")).count())),
        ("parsing_occurrences", J::O(parsing.iter().map(|(k, v)| (k.clone(), jn(*v))).collect())),
        ("sites_with_unvalidated_number_coercion", J::A(unguarded)),
    ]);
    let site_json: Vec<J> = sites
        .into_iter()
        .zip(ids)
        .map(|(site, id)| match site.json {
            J::O(mut items) => {
                items.insert(0, ("id".to_owned(), js(&id)));
                J::O(items)
            }
            other => other,
        })
        .collect();
    let json = jo(vec![
        ("source_sha", js(AWCMS_SHA)),
        (
            "method",
            js("every non-comment call of checkSharedRateLimit / checkAuthRateLimit / checkRateLimit under src/ is parsed (arguments, key template, threshold and window expressions, resolved through their declarations); calls routed through a wrapper function are expanded at the wrapper's callers; each fact cites path, line and an anchor on that line, and the cited files are frozen under source/awcms-impl/. Threshold defaults are the values in source, not the values in any .env file."),
        ),
        ("limiter", J::A(limiter_statements(book)?)),
        ("auth_limiter", {
            let (bucket, facts) = auth_source_bucket(book)?;
            jo(vec![
                ("function", js("checkAuthRateLimit")),
                ("module", js(AUTH_RATE_LIMIT_MODULE)),
                ("statement", js("checks the shared per-source bucket first, then the per-route bucket keyed by client IP, the caller-supplied tenant id and the scope; the first refusal is returned")),
                ("source_bucket", bucket),
                ("facts", J::A(facts)),
                ("order", book.fact_find(AUTH_RATE_LIMIT_MODULE, "if (!source.allowed) {")?),
            ])
        }),
        (
            "not_derivable_from_source",
            jstrs(&[
                "limits applied outside the AWCMS repository (reverse proxy, CDN, WAF) and whether REDIS_URL is configured in a deployment, which decides between the shared and the in-process counter",
                "the effective value of any setting in a deployment: the artifact records the defaults and parsers in source, never an .env value",
                "whether the Turnstile challenge is active in a deployment (it depends on full-online security mode and Turnstile being enabled)",
                "request-size limits (src/lib/security/request-body-limit.ts) are size limits, not rate limits, and are not inventoried here",
            ]),
        ),
        ("summary", summary),
        ("call_sites", J::A(site_json)),
        ("shared_buckets", J::A(shared)),
        ("endpoint_classes", J::A(class_json)),
        (
            "other_abuse_controls",
            J::A(
                controls
                    .into_iter()
                    .map(|c| match c.json {
                        J::O(mut items) => {
                            items.push(("applies_to".to_owned(), applies_to_json(&c.applies_to)));
                            J::O(items)
                        }
                        other => other,
                    })
                    .collect(),
            ),
        ),
        ("claims", J::A(vec![claim])),
    ]);
    let used = book.used_files();
    Ok(RateOutput { json, sites: raws.len(), buckets: bucket_total, classes: classes.len(), used })
}

// ----------------------------------------------------------- credentials ----

const PASSWORD_MODULE: &str = "src/lib/auth/password.ts";
const CREDENTIAL_WORDS: [&str; 11] = ["secret", "token", "password", "api_key", "credential", "verifier", "nonce", "passphrase", "private_key", "code_hash", "state_hash"];

struct CredOutput {
    json: J,
    columns: usize,
    kinds: usize,
    used: Vec<String>,
}

struct Column {
    file: String,
    line: usize,
    table: String,
    column: String,
    ty: String,
    not_null: bool,
    storage: &'static str,
}

fn storage_class(column: &str) -> &'static str {
    if column.contains("ciphertext") || column.contains("encrypted") {
        "encrypted"
    } else if column.contains("hash") {
        "hash"
    } else if column.ends_with("_env_var") {
        "env-var-name"
    } else {
        "plaintext-candidate"
    }
}

/// Table name after `CREATE TABLE [IF NOT EXISTS]` or `ALTER TABLE [IF EXISTS] [ONLY]`.
fn sql_table_of(line: &str) -> Option<String> {
    let t = line.trim_start();
    let rest = t.strip_prefix("CREATE TABLE").or_else(|| t.strip_prefix("ALTER TABLE"))?;
    let mut words = rest.split_whitespace().filter(|w| !matches!(*w, "IF" | "NOT" | "EXISTS" | "ONLY"));
    let name = words.next()?.trim_end_matches('(');
    Some(name.to_owned())
}

/// Credential-like text columns of every migration, classified by name.
fn credential_columns(files: &Files) -> Vec<Column> {
    let mut out = Vec::new();
    for (file, lines) in files {
        if !file.starts_with("sql/") {
            continue;
        }
        let mut table = String::new();
        for (index, line) in lines.iter().enumerate() {
            if let Some(name) = sql_table_of(line) {
                table = name;
            }
            let t = line.trim_start();
            if t.starts_with("--") {
                continue;
            }
            let body = t.strip_prefix("ADD COLUMN IF NOT EXISTS ").or_else(|| t.strip_prefix("ADD COLUMN ")).unwrap_or(t);
            let mut words = body.split_whitespace();
            let (Some(name), Some(ty)) = (words.next(), words.next()) else { continue };
            let ty = ty.trim_end_matches([',', ';']);
            let simple = !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            let textual = ty == "text" || ty == "bytea" || ty.starts_with("varchar") || ty.starts_with("character");
            if simple && textual && CREDENTIAL_WORDS.iter().any(|w| name.contains(w)) {
                out.push(Column {
                    file: file.clone(),
                    line: index + 1,
                    table: table.clone(),
                    column: name.to_owned(),
                    ty: ty.to_owned(),
                    not_null: body.contains("NOT NULL"),
                    storage: storage_class(name),
                });
            }
        }
    }
    out
}

/// A CHECK that pins the stored shape: `col ~ '<regex>'` or `char_length(col) = N`.
fn column_format_check(book: &Book, column: &Column) -> Res<Option<J>> {
    let lines = book.lines(&column.file)?;
    for (index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("--") {
            continue;
        }
        let tilde = format!("{} ~ '", column.column);
        if let Some(at) = line.find(&tilde) {
            let rest = &line[at + tilde.len()..];
            let end = rest.find('\'').ok_or("format check: unterminated regex")?;
            return Ok(Some(jo(vec![("kind", js("regex")), ("pattern", js(&rest[..end])), ("fact", book.fact(&column.file, index + 1, &tilde)?)])));
        }
        let length = format!("char_length({}) = ", column.column);
        if let Some(at) = line.find(&length) {
            let digits: String = line[at + length.len()..].chars().take_while(char::is_ascii_digit).collect();
            return Ok(Some(jo(vec![("kind", js("length")), ("pattern", js(&digits)), ("fact", book.fact(&column.file, index + 1, &length)?)])));
        }
    }
    Ok(None)
}

/// Comment lines that name the column in backticks, and whether the same or the
/// next comment line says PLAINTEXT.
fn column_evidence(book: &Book, column: &Column) -> Res<(Vec<J>, bool)> {
    let lines = book.lines(&column.file)?;
    let needle = format!("`{}`", column.column);
    let mut facts = Vec::new();
    let mut plaintext = false;
    for (index, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("--") && line.contains(&needle) {
            if facts.len() < 3 {
                facts.push(book.fact(&column.file, index + 1, &needle)?);
            }
            let next = lines.get(index + 1).map_or("", String::as_str);
            if line.to_lowercase().contains("plaintext") || (next.trim_start().starts_with("--") && next.to_lowercase().contains("plaintext")) {
                plaintext = true;
            }
        }
    }
    Ok((facts, plaintext))
}

fn column_json(book: &Book, c: &Column, with_evidence: bool) -> Res<J> {
    let mut items = vec![
        ("table", js(&c.table)),
        ("column", js(&c.column)),
        ("type", js(&c.ty)),
        ("not_null", J::B(c.not_null)),
        ("storage", js(c.storage)),
        ("fact", book.fact(&c.file, c.line, &c.column)?),
    ];
    if c.storage == "hash" {
        items.push(("format_check", column_format_check(book, c)?.unwrap_or(J::Null)));
    }
    if with_evidence {
        let (evidence, plaintext) = column_evidence(book, c)?;
        items.push(("comment_evidence", J::A(evidence)));
        items.push(("comment_says_plaintext", J::B(plaintext)));
    }
    Ok(jo(items))
}

/// Body of `export function name(` through the closing `}` at column zero.
fn fn_body(book: &Book, rel: &str, name: &str) -> Res<(usize, Vec<(usize, String)>)> {
    let start = book.find(rel, &format!("export function {name}("))?;
    let lines = book.lines(rel)?;
    let mut body = Vec::new();
    for (index, line) in lines.iter().enumerate().skip(start - 1) {
        body.push((index + 1, line.clone()));
        if line == "}" {
            return Ok((start, body));
        }
    }
    Err(format!("{rel}: function {name} is not closed").into())
}

fn generation_json(book: &Book, rel: &str, name: &str) -> Res<J> {
    let (start, body) = fn_body(book, rel, name)?;
    let text: String = body.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
    let mut facts = vec![book.fact(rel, start, &format!("export function {name}"))?];
    let rb = body.iter().find(|(_, l)| l.contains("randomBytes(") || l.contains("getRandomValues(")).ok_or_else(|| format!("{rel}: {name}: no random source"))?;
    let bytes = if let Some(at) = rb.1.find("randomBytes(") {
        let rest = &rb.1[at + "randomBytes(".len()..];
        let arg = rest[..rest.find(')').ok_or("randomBytes(")?].trim();
        match arg.parse::<i64>() {
            Ok(n) => n,
            Err(_) => const_num(book, rel, arg)?.1,
        }
    } else {
        let line = body.iter().find(|(_, l)| l.contains("new Uint8Array(")).ok_or("getRandomValues without Uint8Array")?;
        let rest = &line.1[line.1.find("new Uint8Array(").ok_or("Uint8Array")? + "new Uint8Array(".len()..];
        rest[..rest.find(')').ok_or("Uint8Array(")?].trim().parse::<i64>().map_err(|_| "Uint8Array length")?
    };
    facts.push(book.fact(rel, rb.0, if rb.1.contains("randomBytes(") { "randomBytes(" } else { "getRandomValues(" })?);
    let encoding = if let Some(line) = body.iter().find(|(_, l)| l.contains(".toString(\"")) {
        let rest = &line.1[line.1.find(".toString(\"").ok_or("toString")? + ".toString(\"".len()..];
        facts.push(book.fact(rel, line.0, ".toString(\"")?);
        rest[..rest.find('"').ok_or("toString(\"")?].to_owned()
    } else if let Some(line) = body.iter().find(|(_, l)| l.contains("base32Encode(")) {
        facts.push(book.fact(rel, line.0, "base32Encode(")?);
        "base32".to_owned()
    } else {
        return Err(format!("{rel}: {name}: encoding not recognised").into());
    };
    let mut prefix = J::Null;
    if let Some(at) = text.find("`${") {
        let ident: String = text[at + 3..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        if ident.ends_with("_PREFIX") {
            let (value, fact) = const_str(book, rel, &ident)?;
            prefix = js(&value);
            facts.push(fact);
        }
    }
    let mut items = vec![("function", js(name)), ("random_bytes", jn(usize::try_from(bytes).unwrap_or(0))), ("encoding", js(&encoding)), ("token_prefix", prefix)];
    if let Some((line, init)) = decl_init(book, rel, "TOKEN_PATTERN")? {
        items.push(("token_pattern", js(&init)));
        facts.push(book.fact(rel, line, "const TOKEN_PATTERN")?);
    }
    items.push(("facts", J::A(facts)));
    Ok(jo(items))
}

fn hash_json(book: &Book, rel: &str, name: &str) -> Res<J> {
    let (start, body) = fn_body(book, rel, name)?;
    let text: String = body.iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>().join("\n");
    let mut facts = vec![book.fact(rel, start, &format!("export function {name}"))?];
    let (anchor, algorithm) = if text.contains("new Bun.CryptoHasher(\"sha256\")") {
        ("new Bun.CryptoHasher(\"sha256\")", "SHA-256")
    } else if text.contains("createHash(\"sha256\")") {
        ("createHash(\"sha256\")", "SHA-256")
    } else {
        return Err(format!("{rel}: {name}: hash algorithm not recognised").into());
    };
    let line = body.iter().find(|(_, l)| l.contains(anchor)).ok_or("hash line")?;
    facts.push(book.fact(rel, line.0, anchor)?);
    if !text.contains("digest(\"hex\")") {
        return Err(format!("{rel}: {name}: digest is not hex").into());
    }
    let mut prefix = String::new();
    if let Some(tick) = line.1.find('`') {
        let after = &line.1[tick + 1..];
        let lit = &after[..after.find("${").unwrap_or(after.len())];
        if lit.is_empty() {
            let ident: String = after[2..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
            let (value, fact) = const_str(book, rel, &ident)?;
            prefix = value;
            facts.push(fact);
        } else {
            prefix = lit.to_owned();
            facts.push(book.fact(rel, line.0, &format!("`{lit}${{"))?);
        }
    }
    Ok(jo(vec![
        ("function", js(name)),
        ("algorithm", js(algorithm)),
        ("output_prefix", js(&prefix)),
        ("digest_encoding", js("hex")),
        ("salted", J::B(false)),
        ("facts", J::A(facts)),
    ]))
}

/// (id, generator module + function, hash module + function, stored in (table, column), extra evidence (file, needle))
type TokenSpec = (&'static str, Option<(&'static str, &'static str)>, (&'static str, &'static str), &'static [(&'static str, &'static str)], &'static [(&'static str, &'static str)]);

const TOKEN_SPECS: [TokenSpec; 12] = [
    (
        "session-token",
        Some(("src/lib/auth/session-token.ts", "generateSessionToken")),
        ("src/lib/auth/session-token.ts", "hashSessionToken"),
        &[("awcms_sessions", "token_hash")],
        &[("src/pages/api/v1/auth/login.ts", "INSERT INTO awcms_sessions"), ("src/modules/identity-access/application/session-lookup.ts", "token_hash")],
    ),
    (
        "machine-credential",
        Some(("src/lib/auth/machine-credential-token.ts", "generateMachineCredentialToken")),
        ("src/lib/auth/machine-credential-token.ts", "hashMachineCredentialToken"),
        &[("awcms_machine_credentials", "token_hash")],
        &[("src/modules/identity-access/application/machine-credential-lookup.ts", "token_hash"), ("src/modules/identity-access/application/machine-credential-directory.ts", "token_hash")],
    ),
    (
        "tenant-selection-token",
        Some(("src/lib/auth/principal-selection-token.ts", "generatePrincipalSelectionToken")),
        ("src/lib/auth/principal-selection-token.ts", "hashPrincipalSelectionToken"),
        &[("awcms_principals", "selection_token_hash")],
        &[],
    ),
    (
        "delegated-access-code",
        Some(("src/lib/auth/delegated-access-code.ts", "generateDelegatedAccessCode")),
        ("src/lib/auth/delegated-access-code.ts", "hashDelegatedAccessCode"),
        &[("awcms_delegated_access_grants", "access_code_hash")],
        &[],
    ),
    (
        "session-handoff-code",
        Some(("src/modules/identity-access/domain/session-handoff.ts", "generateHandoffCode")),
        ("src/modules/identity-access/domain/session-handoff.ts", "hashHandoffCode"),
        &[("awcms_session_handoff_codes", "code_hash")],
        &[],
    ),
    (
        "bff-client-secret",
        None,
        ("src/modules/identity-access/domain/session-handoff.ts", "hashBffClientSecret"),
        &[("awcms_bff_clients", "secret_hash")],
        &[],
    ),
    (
        "password-reset-token",
        Some(("src/lib/auth/reset-token.ts", "generateResetToken")),
        ("src/lib/auth/reset-token.ts", "hashResetToken"),
        &[("awcms_password_reset_tokens", "token_hash")],
        &[],
    ),
    (
        "invitation-token",
        Some(("src/lib/auth/invitation-token.ts", "generateInviteToken")),
        ("src/lib/auth/invitation-token.ts", "hashInviteToken"),
        &[("awcms_invitations", "token_hash")],
        &[],
    ),
    (
        "mfa-challenge-token",
        Some(("src/lib/auth/mfa-challenge-token.ts", "generateChallengeToken")),
        ("src/lib/auth/mfa-challenge-token.ts", "hashChallengeToken"),
        &[("awcms_mfa_challenges", "challenge_token_hash")],
        &[],
    ),
    (
        "mfa-recovery-code",
        Some(("src/lib/auth/mfa-recovery-code.ts", "generateRecoveryCode")),
        ("src/lib/auth/mfa-recovery-code.ts", "hashRecoveryCode"),
        &[("awcms_identity_mfa_recovery_codes", "code_hash"), ("awcms_principal_mfa_recovery_codes", "code_hash")],
        &[],
    ),
    (
        "oauth-state",
        Some(("src/lib/auth/oauth-state-token.ts", "generateOAuthState")),
        ("src/lib/auth/oauth-state-token.ts", "hashOAuthState"),
        &[("awcms_oidc_auth_requests", "state_hash")],
        &[],
    ),
    (
        "newsletter-subscription-token",
        Some(("src/modules/newsletter/domain/subscription-token.ts", "generateSubscriptionToken")),
        ("src/modules/newsletter/domain/subscription-token.ts", "hashSubscriptionToken"),
        &[("awcms_newsletter_subscribers", "confirmation_token_hash"), ("awcms_newsletter_subscribers", "unsubscribe_token_hash")],
        &[],
    ),
];

/// (id, module, encrypt function, key resolver, key env var, stored in)
type CipherSpec = (&'static str, &'static str, &'static str, &'static str, &'static [(&'static str, &'static str)]);

const CIPHER_SPECS: [CipherSpec; 2] = [
    (
        "mfa-totp-secret",
        "src/lib/auth/mfa-secret-crypto.ts",
        "encryptMfaSecret",
        "resolveMfaEncryptionKey",
        &[("awcms_identity_mfa_factors", "secret_ciphertext"), ("awcms_principal_mfa_factors", "secret_ciphertext")],
    ),
    (
        "sso-client-secret",
        "src/lib/auth/sso-credential-crypto.ts",
        "encryptSsoClientSecret",
        "resolveSsoEncryptionKey",
        &[("awcms_auth_providers", "client_secret_ciphertext")],
    ),
];

fn cipher_json(book: &Book, columns: &[Column], spec: &CipherSpec) -> Res<J> {
    let (id, rel, encrypt, key_fn, stored) = *spec;
    let algorithm = const_str(book, rel, "ALGORITHM")?;
    let version = const_str(book, rel, "FORMAT_VERSION")?;
    let (_, iv, iv_fact) = const_num(book, rel, "IV_BYTE_LENGTH")?;
    let (start, body) = fn_body(book, rel, encrypt)?;
    let mut layout = Vec::new();
    let mut in_array = false;
    for (_, line) in &body {
        let t = line.trim();
        if t.starts_with("return [") {
            in_array = true;
            continue;
        }
        if t.starts_with("].join(\":\")") {
            break;
        }
        if in_array {
            let t = t.trim_end_matches(',');
            if t == "FORMAT_VERSION" {
                layout.push(format!("version:{}", version.0));
            } else if let Some((var, rest)) = t.split_once(".toString(\"") {
                layout.push(format!("{var}:{}", rest.trim_end_matches("\")")));
            } else {
                return Err(format!("{rel}: {encrypt}: unrecognised layout element `{t}`").into());
            }
        }
    }
    if layout.is_empty() {
        return Err(format!("{rel}: {encrypt}: no layout").into());
    }
    let (key_start, key_body) = fn_body(book, rel, key_fn)?;
    let env_line = key_body.iter().find(|(_, l)| l.contains("env.AUTH_")).ok_or("cipher key env")?;
    let env = &env_line.1[env_line.1.find("env.").ok_or("env.")? + 4..];
    let env: String = env.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
    let length_line = key_body.iter().find(|(_, l)| l.contains("key.length === KEY_BYTE_LENGTH")).ok_or("cipher key length check")?;
    let (_, key_bytes, key_const_fact) = const_num(book, rel, "KEY_BYTE_LENGTH")?;
    let decode_line = key_body.iter().find(|(_, l)| l.contains("Buffer.from(raw, \"base64\")")).ok_or("cipher key encoding")?;
    let mut stores = Vec::new();
    for (table, column) in stored {
        let c = columns.iter().find(|c| c.table == *table && c.column == *column).ok_or_else(|| format!("migration column not found: {table}.{column}"))?;
        if c.storage != "encrypted" {
            return Err(format!("{table}.{column} is not an encrypted column").into());
        }
        stores.push(column_json(book, c, false)?);
    }
    Ok(jo(vec![
        ("id", js(id)),
        ("algorithm", js(&algorithm.0)),
        ("iv_bytes", jn(usize::try_from(iv).unwrap_or(0))),
        ("layout", jstrs(&layout)),
        ("key_env", js(&env)),
        ("key_bytes", jn(usize::try_from(key_bytes).unwrap_or(0))),
        ("key_encoding", js("base64")),
        ("stored_in", J::A(stores)),
        ("facts", J::A(vec![algorithm.1, version.1, iv_fact, book.fact(rel, start, &format!("export function {encrypt}"))?, book.fact(rel, key_start, &format!("export function {key_fn}"))?, book.fact(rel, env_line.0, &format!("env.{env}"))?, book.fact(rel, length_line.0, "key.length === KEY_BYTE_LENGTH")?, key_const_fact, book.fact(rel, decode_line.0, "Buffer.from(raw, \"base64\")")?])),
    ]))
}

fn find_hits(book: &Book, prefix: &str, patterns: &[&str], skip: &[&str]) -> Res<Vec<J>> {
    let mut hits = Vec::new();
    for (file, lines) in book.files {
        if !file.starts_with(prefix) || skip.contains(&file.as_str()) {
            continue;
        }
        for (index, line) in lines.iter().enumerate() {
            if !is_code_line(line) {
                continue;
            }
            for pattern in patterns {
                if line.contains(pattern) {
                    hits.push(book.fact(file, index + 1, pattern)?);
                }
            }
        }
    }
    Ok(hits)
}

fn password_json(book: &Book, columns: &[Column]) -> Res<J> {
    // Parameters come from the PHC prefix of the constant dummy hash and the
    // doc comment that states the library defaults; the salt and digest of that
    // constant are never read into the artifact.
    let phc_line = book.find(LOGIN_POLICY, "\"$argon2id$v=")?;
    let text = &book.lines(LOGIN_POLICY)?[phc_line - 1];
    let rest = &text[text.find("$argon2id$v=").ok_or("phc")? + "$argon2id$v=".len()..];
    let (v, rest) = rest.split_once("$m=").ok_or("phc m")?;
    let (m, rest) = rest.split_once(",t=").ok_or("phc t")?;
    let (t, rest) = rest.split_once(",p=").ok_or("phc p")?;
    let p: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let prefix = format!("$argon2id$v={v}$m={m},t={t},p={p}$");
    let doc_anchor = format!("argon2id, v={v}, m={m}, t={t}, p={p}");
    let num = |s: &str| s.parse::<usize>().map_err(|_| format!("phc: `{s}` is not a number"));
    let mut stored = Vec::new();
    for (table, column) in [("awcms_identities", "password_hash"), ("awcms_principals", "password_hash")] {
        let c = columns.iter().find(|c| c.table == table && c.column == column).ok_or_else(|| format!("migration column not found: {table}.{column}"))?;
        stored.push(column_json(book, c, false)?);
    }
    let mut verified = Vec::new();
    let mut written = Vec::new();
    for (file, lines) in book.files {
        if !file.starts_with("src/") || file == PASSWORD_MODULE {
            continue;
        }
        for (index, line) in lines.iter().enumerate() {
            let t = line.trim();
            if !is_code_line(line) || t.starts_with("import") || t.starts_with("export ") || t.starts_with("verifyPassword") && t.ends_with(',') || t == "hashPassword," {
                continue;
            }
            for (needle, bucket) in [("verifyPasswordOrDummy(", 0), ("verifyPassword(", 0), ("hashPassword(", 1)] {
                if line.contains(needle) && !line.contains(&format!("function {}", needle.trim_end_matches('('))) {
                    let fact = book.fact(file, index + 1, t)?;
                    if bucket == 0 { verified.push(fact) } else { written.push(fact) }
                    break;
                }
            }
        }
    }
    let legacy_patterns = ["bcrypt", "scrypt", "pbkdf2", "createHash(\"md5\")", "createHash(\"sha1\")"];
    let rehash_patterns = ["needsRehash", "rehash"];
    Ok(jo(vec![
        ("algorithm", js("argon2id")),
        ("algorithm_basis", js("hashPassword calls Bun.password.hash(password) with no options, so the algorithm and cost are the runtime defaults; the defaults are stated in the doc comment cited below and match the PHC prefix of the constant dummy hash")),
        ("library", js("Bun.password (runtime built-in)")),
        ("hash_call", book.fact_find(PASSWORD_MODULE, "Bun.password.hash(password)")?),
        ("verify_call", book.fact_find(PASSWORD_MODULE, "Bun.password.verify(password, hash)")?),
        (
            "parameters",
            jo(vec![
                ("version", jn(num(v)?)),
                ("memory_kib", jn(num(m)?)),
                ("iterations", jn(num(t)?)),
                ("parallelism", jn(num(&p)?)),
                ("encoding", js("PHC string")),
                ("facts", J::A(vec![book.fact(LOGIN_POLICY, phc_line, &prefix)?, book.fact_find(LOGIN_POLICY, &doc_anchor)?])),
            ]),
        ),
        ("stored_in", J::A(stored)),
        ("dummy_hash", jo(vec![("purpose", js("verifyPasswordOrDummy verifies against a constant hash when no stored hash exists, so both branches pay one verify")), ("phc_prefix", js(&prefix)), ("constant", book.fact_find(LOGIN_POLICY, "const DUMMY_PASSWORD_HASH")?), ("use", book.fact_find(LOGIN_POLICY, "await verifyPassword(password, DUMMY_PASSWORD_HASH);")?)])),
        ("verified_at", J::A(verified)),
        ("hash_written_at", J::A(written)),
        (
            "legacy_format_search",
            jo(vec![
                ("scope", js("non-comment lines of src/**/*.ts and *.astro")),
                ("patterns", jstrs(&legacy_patterns)),
                ("hits", J::A(find_hits(book, "src/", &legacy_patterns, &[])?)),
            ]),
        ),
        (
            "rehash_on_login_search",
            jo(vec![
                ("scope", js("non-comment lines of src/**/*.ts and *.astro")),
                ("patterns", jstrs(&rehash_patterns)),
                ("hits", J::A(find_hits(book, "src/", &rehash_patterns, &[])?)),
            ]),
        ),
    ]))
}

fn build_credentials(book: &Book) -> Res<CredOutput> {
    let columns = credential_columns(book.files);
    let password = password_json(book, &columns)?;

    let mut tokens = Vec::new();
    for (id, generator, hasher, stored, evidence) in TOKEN_SPECS {
        let generation = match generator {
            Some((rel, name)) => generation_json(book, rel, name)?,
            None => J::Null,
        };
        let hash = hash_json(book, hasher.0, hasher.1)?;
        let mut stores = Vec::new();
        for (table, column) in stored {
            let c = columns.iter().find(|c| c.table == *table && c.column == *column).ok_or_else(|| format!("migration column not found: {table}.{column}"))?;
            if c.storage != "hash" {
                return Err(format!("{table}.{column} is not a hash column").into());
            }
            stores.push(column_json(book, c, false)?);
        }
        let mut proofs = Vec::new();
        for (file, needle) in evidence {
            proofs.push(book.fact_find(file, needle)?);
        }
        tokens.push(jo(vec![("id", js(id)), ("generation", generation), ("hash", hash), ("stored_in", J::A(stores)), ("evidence", J::A(proofs))]));
    }

    let mut ciphers = Vec::new();
    for spec in &CIPHER_SPECS {
        ciphers.push(cipher_json(book, &columns, spec)?);
    }

    let mut column_json_list = Vec::new();
    let mut plaintext = Vec::new();
    for c in &columns {
        column_json_list.push(jo(vec![("file", js(&c.file)), ("detail", column_json(book, c, false)?)]));
        if c.storage == "plaintext-candidate" {
            plaintext.push(jo(vec![("file", js(&c.file)), ("detail", column_json(book, c, true)?)]));
        }
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for c in &columns {
        *counts.entry(c.storage).or_default() += 1;
    }
    let password_columns = ["password", "passwd", "plain_password", "password_plain"];
    let mut plain_password_hits = Vec::new();
    for (file, lines) in book.files {
        if !file.starts_with("sql/") {
            continue;
        }
        for (index, line) in lines.iter().enumerate() {
            let t = line.trim_start();
            if t.starts_with("--") {
                continue;
            }
            let body = t.strip_prefix("ADD COLUMN IF NOT EXISTS ").or_else(|| t.strip_prefix("ADD COLUMN ")).unwrap_or(t);
            if let Some(first) = body.split_whitespace().next()
                && password_columns.contains(&first)
            {
                plain_password_hits.push(book.fact(file, index + 1, first)?);
            }
        }
    }
    let json = jo(vec![
        ("source_sha", js(AWCMS_SHA)),
        (
            "method",
            js("formats and parameters only, read from the pinned implementation and migrations: hash and generator functions are located by name and their bodies parsed; credential-like columns are every text/bytea column of sql/*.sql whose name contains a credential word, classified by name (hash, encrypted, env-var-name, plaintext-candidate); each fact cites path, line and an anchor on that line and the cited files are frozen under source/awcms-impl/. No hash value, key, secret or fixture credential is copied; the only value taken from the constant dummy hash is its PHC parameter prefix."),
        ),
        (
            "not_derivable_from_source",
            jstrs(&[
                "which hash, if any, a given production row actually holds: the artifact records what the code writes, not database contents",
                "whether passwords stored before a library default changed still verify under older parameters: Argon2 PHC strings carry their own parameters and no rehash-on-login code exists (see rehash_on_login_search)",
                "tables created outside sql/*.sql (none is assumed) and credential values held in environment variables or external secret stores",
            ]),
        ),
        ("password", password),
        ("bearer_tokens", J::A(tokens)),
        ("encrypted_secrets", J::A(ciphers)),
        ("credential_columns", J::A(column_json_list)),
        ("plaintext_candidates", J::A(plaintext)),
        ("plaintext_password_column_search", jo(vec![("column_names", jstrs(&password_columns)), ("hits", J::A(plain_password_hits))])),
        ("summary", jo(vec![("columns", jn(columns.len())), ("by_storage", J::O(counts.iter().map(|(k, v)| ((*k).to_owned(), jn(*v))).collect())), ("bearer_token_kinds", jn(TOKEN_SPECS.len())), ("encrypted_secret_kinds", jn(CIPHER_SPECS.len()))])),
    ]);
    let n_columns = columns.len();
    Ok(CredOutput { json, columns: n_columns, kinds: TOKEN_SPECS.len() + CIPHER_SPECS.len() + 1, used: book.used_files() })
}

fn with_sources(json: J, used: &[String]) -> J {
    match json {
        J::O(mut items) => {
            items.push(("frozen_sources".to_owned(), jstrs(used)));
            J::O(items)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_module_imports_without_shared_contract_import() {
        let source = r#"
import type { ModuleDescriptor } from "./_shared/module-contract";
import { loggingModule } from "./logging/module";
import { tenantAdminModule } from "./tenant-admin/module";
"#;
        assert_eq!(
            parse_module_imports(source),
            vec!["logging", "tenant-admin"]
        );
    }

    #[test]
    fn parses_consumed_and_committed_path_blocks() {
        let source = r#"
export const CONSUMED_PATHS = {
  "/api/v1/a": "a",
  "/api/v1/b": "b"
};
export const COMMITTED_PATHS = {
  "/api/v1/c": "c"
};
export const CONSUMER_PATHS = {};
"#;
        assert_eq!(
            parse_record_paths(
                source,
                "export const CONSUMED_PATHS",
                "export const COMMITTED_PATHS"
            )
            .unwrap(),
            vec!["/api/v1/a", "/api/v1/b"]
        );
        assert_eq!(
            parse_record_paths(
                source,
                "export const COMMITTED_PATHS",
                "export const CONSUMER_PATHS"
            )
            .unwrap(),
            vec!["/api/v1/c"]
        );
    }

    #[test]
    fn parses_package_scripts_and_expands_chains() {
        let package = "{\n  \"scripts\": {\n    \"a:check\": \"bun scripts/a.ts\",\n    \"build\": \"bun run build:x && bun run a:check\",\n    \"build:x\": \"echo \\\"hi\\\"\",\n    \"check\": \"bun run build && bun run lint\",\n    \"lint\": \"prettier --check .\"\n  },\n  \"x\": 1\n}\n";
        let scripts = parse_package_scripts(package).unwrap();
        assert_eq!(scripts.len(), 5);
        assert_eq!(scripts[2].cmd, "echo \"hi\"");
        let map: BTreeMap<String, String> = scripts.iter().map(|s| (s.name.clone(), s.cmd.clone())).collect();
        let mut chain = BTreeSet::new();
        expand_script("check", &map, &mut chain);
        assert_eq!(chain.into_iter().collect::<Vec<_>>(), vec!["a:check", "build", "build:x", "check", "lint"]);
        assert!(parse_package_scripts("{}").is_err());
    }

    #[test]
    fn gate_like_names() {
        for name in ["check", "api:spec:check", "release:verify", "lint", "typecheck", "test", "test:e2e", "build"] {
            assert!(gate_like_name(name), "{name}");
        }
        for name in ["dev", "db:migrate", "build:clean", "test:e2e:install", "push:vapid:generate"] {
            assert!(!gate_like_name(name), "{name}");
        }
    }

    #[test]
    fn extracts_bun_commands_from_run_text() {
        let text = "bun run check && bun run jobs:crontab:check\nbun scripts/foo-check.ts --x\nbun run i18n:";
        assert_eq!(bun_run_names(text), vec!["check", "jobs:crontab:check", "i18n"]);
        assert_eq!(bun_script_files(text), vec!["scripts/foo-check.ts"]);
    }

    #[test]
    fn workflow_events_jobs_and_runs() {
        let yml = "name: Demo\non:\n  push:\n    branches: [main]\n  pull_request:\n    branches: [main]\njobs:\n  quality:\n    name: Q\n    steps:\n      - run: bun run check\n      - name: x\n        run: |\n          # comment\n          bun test\n          bun run lint\n  release:\n    steps:\n      - run: bun run release:verify\n";
        let wf = parse_workflow(".github/workflows/demo.yml", yml);
        assert_eq!(wf.name, "Demo");
        assert_eq!(wf.events, vec!["pull_request", "push:branches"]);
        assert!(wf.pr_blocking());
        assert_eq!(wf.jobs.len(), 2);
        assert_eq!(wf.jobs[0].name, "Q");
        assert_eq!(wf.jobs[0].runs.len(), 2);
        assert!(wf.jobs[0].runs[1].contains("bun test"));
        assert!(!wf.jobs[0].runs[1].contains("comment"));

        let tag_only = "on:\n  push:\n    tags:\n      - \"v*\"\n  workflow_dispatch: {}\njobs:\n  a:\n    steps: []\n";
        let wf = parse_workflow("x.yml", tag_only);
        assert_eq!(wf.events, vec!["push:tags", "workflow_dispatch"]);
        assert!(!wf.pr_blocking());
    }

    #[test]
    fn route_naming_and_patterns() {
        assert_eq!(route_of("blog/[tenantCode]/[slug].ts"), "/blog/[tenantCode]/[slug]");
        assert_eq!(route_of("admin/index.astro"), "/admin");
        assert_eq!(route_of("index.astro"), "/index");
        assert!(pattern_matches_route("/api/v1/comments/:id", "/api/v1/comments/[id]"));
        assert!(!pattern_matches_route("/api/v1/comments/:id", "/api/v1/comments"));
        assert!(!pattern_matches_route("/api/v1/comments/:id", "/api/v1/comments/reply"));
    }

    #[test]
    fn splits_handlers_and_ignores_imports() {
        let source = "import { x } from \"a\";\nconst helper = 1;\nexport const GET = async () => {};\nexport const prerender = false;\nexport const POST: APIRoute = async () => {};\n";
        let (handlers, preamble) = split_handlers(source);
        assert_eq!(handlers.iter().map(|h| h.0.as_str()).collect::<Vec<_>>(), vec!["GET", "POST"]);
        assert!(handlers[0].1.contains("prerender"));
        assert_eq!(preamble, "const helper = 1;");
    }

    #[test]
    fn parses_session_free_registry_entries() {
        let source = "export const SESSION_FREE_BODY_ENDPOINTS: readonly X[] = [\n  {\n    method: \"POST\",\n    pattern: \"/api/v1/a/:id\",\n    reason:\n      \"Because.\"\n  },\n  {\n    method: \"PATCH\",\n    pattern: \"/api/v1/b\",\n    reason: \"Inline.\"\n  }\n];\nconst BODY_METHODS = 1;\n";
        let free = parse_session_free(source).unwrap();
        assert_eq!(free.len(), 2);
        assert_eq!((free[0].method.as_str(), free[0].pattern.as_str(), free[0].reason.as_str()), ("POST", "/api/v1/a/:id", "Because."));
        assert_eq!(free[1].reason, "Inline.");
    }

    #[test]
    fn parses_openapi_operations() {
        let yaml = "openapi: 3.1.0\npaths:\n  /api/v1/x/{id}:\n    get:\n      operationId: getX\n      security: []\n      responses:\n        \"200\":\n          $ref: \"#/components/responses/Ok\"\n        \"404\":\n          description: no\n    post:\n      operationId: postX\n      requestBody:\n        content:\n          application/json:\n            schema:\n              $ref: '#/components/schemas/XBody'\n      responses:\n        \"201\":\n          content:\n            application/json:\n              schema:\n                $ref: \"#/components/schemas/X\"\ncomponents:\n  a: b\n";
        let ops = parse_openapi_ops(yaml);
        assert_eq!(ops.len(), 2);
        assert_eq!((ops[0].method.as_str(), ops[0].path.as_str(), ops[0].security), ("GET", "/api/v1/x/[id]", "none"));
        assert_eq!(ops[0].responses, vec!["200 #/components/responses/Ok", "404 -"]);
        assert_eq!(ops[1].security, "global");
        assert_eq!(ops[1].request_ref.as_deref(), Some("#/components/schemas/XBody"));
        assert_eq!(ops[1].responses, vec!["201 #/components/schemas/X"]);
    }

    #[test]
    fn extracts_test_asserted_exchanges() {
        let source = "describe(\"OPTIONS /api/v1/a\", () => {\n  test(\"no origin gets 204\", async () => {\n    expect(response.status).toBe(204);\n  });\n  test(\"unrelated\", () => {\n    expect(1).toBe(1);\n  });\n});\ndescribe(\"other\", () => {\n  test(\"POST /api/v1/b/{id} is 410 Gone\", async () => {\n    expect(r.status).toBe(410);\n    expect(body.error.code).toBe(\"ENDPOINT_RETIRED\");\n  });\n});\n";
        let found = extract_exchanges("tests/x.test.ts", source);
        assert_eq!(found.len(), 2);
        assert_eq!((found[0].method.as_str(), found[0].path.as_str(), found[0].origin), ("OPTIONS", "/api/v1/a", "describe-title"));
        assert_eq!(found[0].statuses, vec![204]);
        assert_eq!((found[1].path.as_str(), found[1].origin), ("/api/v1/b/{id}", "test-title"));
        assert_eq!(found[1].codes, vec!["ENDPOINT_RETIRED"]);
        assert_eq!(parse_operation("GET /api/v1/workflows/tasks: 200 while enabled"), Some(("GET".to_owned(), "/api/v1/workflows/tasks".to_owned())));
        assert_eq!(parse_operation("FORGET /api/x"), None);
    }

    #[test]
    fn json_rendering_is_stable_and_escaped() {
        let value = jo(vec![
            ("a", js("x\"y\n")),
            ("b", J::A(vec![jo(vec![("k", jn(1))]), jo(vec![("k", J::Null)])])),
            ("c", jstrs(&["p", "q"])),
        ]);
        let mut text = String::new();
        render_json(&value, 0, &mut text);
        assert_eq!(text, "{\n  \"a\": \"x\\\"y\\n\",\n  \"b\": [\n    {\"k\": 1},\n    {\"k\": null}\n  ],\n  \"c\": [\"p\", \"q\"]\n}");
    }

    fn one_file(rel: &str, text: &str) -> Files {
        let mut files = Files::new();
        files.insert(rel.to_owned(), text.lines().map(str::to_owned).collect());
        files
    }

    #[test]
    fn scans_groups_and_splits_top_level() {
        let lines: Vec<String> = ["call(a, \"x,)\", {", "  k: f(1, 2), // c, )", "}, [3, 4]);", "tail"].iter().map(|s| (*s).to_owned()).collect();
        let (group, span) = call_group(&lines, 0, "call").unwrap();
        assert_eq!(span, 2);
        let args = split_top(group_inner(&group), ',');
        assert_eq!(args.len(), 4);
        assert_eq!(args[1], "\"x,)\"");
        assert_eq!(config_props(&args[2]), vec![("k".to_owned(), "f(1, 2)".to_owned())]);
        assert_eq!(split_pair("a: b ? c : d", ':'), Some(("a".to_owned(), "b ? c : d".to_owned())));
        assert_eq!(strip_outer_parens("((a) + (b))"), "(a) + (b)");
        assert_eq!(strip_outer_parens("(a) + (b)"), "(a) + (b)");
    }

    #[test]
    fn numeric_helpers() {
        assert_eq!(lit_product("60 * 60 * 1000"), Some(3_600_000));
        assert_eq!(lit_product("50_000"), Some(50_000));
        assert_eq!(lit_product("X * 2"), None);
        assert_eq!(parse_guard("Number.isFinite(A) && A > 0 ? A : 120"), Some(("A".to_owned(), 120)));
        assert_eq!(parse_guard("Number.isFinite(A) && B > 0 ? A : 120"), None);
        assert_eq!(norm("  a\n   b ,"), "a b");
        assert_eq!(env_name_of("process.env.X_Y"), Some("X_Y".to_owned()));
    }

    #[test]
    fn resolves_settings_through_their_declarations() {
        let source = "const MAX = parsePositiveIntSetting(\n  process.env.A_MAX,\n  5,\n  \"A_MAX\"\n);\nconst RAW = Number(\n  process.env.B_MAX ?? 120\n);\nconst WIN_MS = 60 * 60 * 1000;\nconst G = Number.isFinite(RAW) && RAW > 0 ? RAW : 120;\n";
        let files = one_file("src/x.ts", source);
        let book = Book::new(&files);
        let max = resolve_value(&book, "src/x.ts", "MAX").unwrap();
        assert_eq!((max.env.as_deref(), max.default, max.parsing), (Some("A_MAX"), 5, "parsePositiveIntSetting"));
        let raw = resolve_value(&book, "src/x.ts", "RAW").unwrap();
        assert_eq!((raw.env.as_deref(), raw.default, raw.parsing), (Some("B_MAX"), 120, "Number-coercion-unguarded"));
        let guarded = resolve_value(&book, "src/x.ts", "Number.isFinite(RAW) && RAW > 0 ? RAW : 120").unwrap();
        assert_eq!((guarded.parsing, guarded.fallback), ("Number-coercion-guarded", Some(120)));
        let window = resolve_window(&book, "src/x.ts", "WIN_MS").unwrap();
        assert_eq!((window.parsing, window.default), ("literal", 3600));
        assert!(resolve_value(&book, "src/x.ts", "MISSING").is_err());
        assert!(book.fact("src/x.ts", 2, "process.env.A_MAX").is_ok());
        assert!(book.fact("src/x.ts", 2, "process.env.OTHER").is_err());
    }

    #[test]
    fn splits_key_templates_and_finds_handlers() {
        let (prefix, exprs) = template_parts("`comments:list:${clientIp}`").unwrap();
        assert_eq!((prefix.as_str(), exprs), ("comments:list:", vec!["clientIp".to_owned()]));
        assert!(template_parts("\"plain\"").is_err());
        assert!(classify_key_expr("somethingElse").is_err());
        let lines: Vec<String> = ["async function helper() {", "  limit();", "}", "export const GET = async () => {", "  helper();", "};", "export const POST = async () => {};"].iter().map(|s| (*s).to_owned()).collect();
        assert_eq!(enclosing_handler(&lines, 4).as_deref(), Some("GET"));
        assert_eq!(enclosing_handler(&lines, 1).as_deref(), Some("GET"));
        assert_eq!(enclosing_params(&lines, 4).map(|(n, _)| n), Some("helper".to_owned()));
    }

    #[test]
    fn classifies_credential_columns_by_name() {
        let sql = "CREATE TABLE IF NOT EXISTS awcms_t (\n  id uuid,\n  token_hash text NOT NULL,\n  secret_ciphertext text,\n  nonce text NOT NULL,\n  note text,\n  CONSTRAINT x CHECK (token_hash ~ '^k-sha256:[0-9a-f]{64}$')\n);\nALTER TABLE awcms_t\n  ADD COLUMN IF NOT EXISTS selection_token_hash text;\n-- password text\n";
        let files = one_file("sql/001_x.sql", sql);
        let columns = credential_columns(&files);
        let seen: Vec<(&str, &str, &str, bool)> = columns.iter().map(|c| (c.table.as_str(), c.column.as_str(), c.storage, c.not_null)).collect();
        assert_eq!(
            seen,
            vec![
                ("awcms_t", "token_hash", "hash", true),
                ("awcms_t", "secret_ciphertext", "encrypted", false),
                ("awcms_t", "nonce", "plaintext-candidate", true),
                ("awcms_t", "selection_token_hash", "hash", false),
            ]
        );
        let book = Book::new(&files);
        let check = column_format_check(&book, &columns[0]).unwrap().unwrap();
        let mut text = String::new();
        render_json(&check, 0, &mut text);
        assert!(text.contains("^k-sha256:[0-9a-f]{64}$"));
    }

    #[test]
    fn sha256_uses_standard_digest() {
        let path = env::temp_dir().join(format!("awbms-vg01-sha-{}", std::process::id()));
        fs::write(&path, b"abc").unwrap();
        let digest = sha256_file(&path).unwrap();
        let _ = fs::remove_file(&path);
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}

use std::{
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

    write_manifest(output)?;

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

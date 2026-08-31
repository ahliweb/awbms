use std::{
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

    write_manifest(output)?;

    println!(
        "VG-01 static inventory generated: {EXPECTED_MODULES} modules, {EXPECTED_MIGRATIONS} migrations, {EXPECTED_CONSUMED_PATHS} consumed paths, {EXPECTED_COMMITTED_PATHS} committed paths"
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

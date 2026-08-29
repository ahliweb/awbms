use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::args().nth(1).unwrap_or_else(|| ".".to_owned());
    let root = Path::new(&root);
    let violations = scan(root)?;
    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("VG-15 violation: {violation}");
        }
        return Err(format!(
            "VG-15 failed with {} repository-independence violation(s)",
            violations.len()
        )
        .into());
    }
    println!("VG-15 repository-independence check passed");
    Ok(())
}

fn scan(root: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut violations = Vec::new();
    if root.join(".gitmodules").exists() {
        violations.push(
            ".gitmodules is present; AWBMS must not depend on AWCMS through a submodule".to_owned(),
        );
    }

    let mut files = Vec::new();
    collect_relevant(root, root, &mut files)?;
    files.sort();
    for file in files {
        let text = match fs::read_to_string(&file) {
            Ok(text) => text,
            Err(_) => continue,
        };
        let relative = file
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        for pattern in forbidden_patterns() {
            if text.contains(pattern) {
                violations.push(format!(
                    "{relative} contains prohibited runtime/source coupling marker `{pattern}`"
                ));
            }
        }
    }
    Ok(violations)
}

fn collect_relevant(
    root: &Path,
    current: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries: Vec<_> = fs::read_dir(current)?.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        if relative == ".git"
            || relative.starts_with(".git/")
            || relative == ".legacy"
            || relative.starts_with(".legacy/")
            || relative.starts_with("target/")
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_relevant(root, &path, out)?;
            continue;
        }
        if is_relevant_file(&relative) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_relevant_file(relative: &str) -> bool {
    relative == "Cargo.toml"
        || relative.ends_with("/Cargo.toml")
        || (relative.ends_with(".rs") && relative.starts_with("crates/"))
        || relative == "Dockerfile"
        || relative.starts_with("Dockerfile.")
        || relative.ends_with("/Dockerfile")
        || relative.ends_with("/Dockerfile.production")
        || relative.ends_with("docker-compose.yml")
        || relative.ends_with("docker-compose.yaml")
        || relative.starts_with("deploy/")
        || relative.starts_with("config/")
}

fn forbidden_patterns() -> &'static [&'static str] {
    &[
        "github.com/ahliweb/awcms",
        "git@github.com:ahliweb/awcms",
        "../awcms",
        "../../awcms",
        "/awcms/src/",
        "awcms-astro/src/",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("awbms-vg15-{name}-{suffix}"));
        fs::create_dir_all(root.join("crates/a/src")).unwrap();
        root
    }

    #[test]
    fn permits_documented_evidence_but_rejects_cargo_coupling() {
        let root = temp_root("cargo");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(
            root.join("docs/reference.md"),
            "https://github.com/ahliweb/awcms",
        )
        .unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[dependencies]\nlegacy = { git = \"https://github.com/ahliweb/awcms\" }\n",
        )
        .unwrap();
        let violations = scan(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn clean_runtime_source_passes() {
        let root = temp_root("clean");
        fs::write(root.join("Cargo.toml"), "[workspace]\nmembers=[]\n").unwrap();
        fs::write(root.join("crates/a/src/lib.rs"), "pub fn ok() {}\n").unwrap();
        let violations = scan(&root).unwrap();
        let _ = fs::remove_dir_all(&root);
        assert!(violations.is_empty());
    }
}

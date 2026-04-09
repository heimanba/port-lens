//! Walk upward from a process cwd to find a recognizable project root.

use std::path::{Path, PathBuf};

/// True if `dir` looks like the root of an app/repo we care about.
pub fn has_project_markers(dir: &Path) -> bool {
    dir.join("package.json").is_file()
        || dir.join("Cargo.toml").is_file()
        || dir.join("go.mod").is_file()
        || dir.join("pyproject.toml").is_file()
        || dir.join("Gemfile").is_file()
        || dir.join("manage.py").is_file()
}

/// Nearest ancestor of `cwd` (including `cwd`) that contains project markers.
pub fn resolve_project_root(cwd: &Path) -> Option<PathBuf> {
    let mut cur = Some(cwd);
    while let Some(p) = cur {
        if has_project_markers(p) {
            return Some(p.to_path_buf());
        }
        cur = p.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolve_finds_parent_package_json() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_else(|e| panic!("system time before UNIX epoch: {e:?}"));
        let base = std::env::temp_dir().join(format!("port-lens-pr-{nanos}"));
        let nested = base.join("apps").join("svc").join("bin");
        fs::create_dir_all(&nested).unwrap_or_else(|e| {
            panic!("create_dir_all {}: {e}", nested.display());
        });
        let pkg = base.join("package.json");
        fs::write(&pkg, "{}").unwrap_or_else(|e| panic!("write {}: {e}", pkg.display()));
        assert_eq!(
            resolve_project_root(&nested).as_deref(),
            Some(base.as_path())
        );
        let _ = fs::remove_dir_all(&base);
    }
}

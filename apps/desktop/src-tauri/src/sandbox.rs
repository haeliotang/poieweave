// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Path resolution for the app-owned workspace sandbox.
//!
//! The host, not the webview, decides where files may live. Every path coming
//! from the frontend is relative, rejects parent/root/prefix components, and is
//! resolved under a single app-data root. Reads and writes are size-limited.

use std::path::{Component, Path, PathBuf};

pub const MAX_FILE_BYTES: u64 = 1_048_576;

pub fn resolve(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative = relative.trim();
    if relative.is_empty() {
        return Err("empty sandbox path".into());
    }
    let mut output = root.to_path_buf();
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => output.push(part),
            _ => return Err(format!("sandbox path must be relative and stay inside the root: {relative:?}")),
        }
    }
    Ok(output)
}

fn reject_symlinks(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = resolve(root, relative)?;
    let mut current = root.to_path_buf();
    for component in Path::new(relative).components() {
        let Component::Normal(part) = component else {
            return Err(format!("sandbox path must be relative: {relative:?}"));
        };
        current.push(part);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("sandbox path must not contain symlinks: {}", current.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{}: {error}", current.display())),
        }
    }
    Ok(path)
}

/// Resolve `relative` beneath `base_relative` but allow `..` only when the
/// canonical result stays inside `root`. Symlinks are rejected.
pub fn resolve_inside(root: &Path, base_relative: &str, relative: &str) -> Result<PathBuf, String> {
    let base = resolve(root, base_relative)?;
    let candidate = base.join(relative);
    let root_canon = std::fs::canonicalize(root).map_err(|error| format!("{}: {error}", root.display()))?;
    let candidate_canon =
        std::fs::canonicalize(&candidate).map_err(|error| format!("{}: {error}", candidate.display()))?;
    if !candidate_canon.starts_with(&root_canon) {
        return Err(format!("sandbox path escapes the root: {relative:?}"));
    }
    let mut current = root_canon.clone();
    for component in candidate_canon.strip_prefix(&root_canon).map_err(|_| "outside root".to_string())?.components() {
        current.push(component);
        if let Ok(metadata) = std::fs::symlink_metadata(&current) {
            if metadata.file_type().is_symlink() {
                return Err(format!("sandbox path must not contain symlinks: {}", current.display()));
            }
        }
    }
    Ok(candidate_canon)
}

/// Read a file resolved relative to another sandbox path while keeping it
/// inside the root. Used for a timed scenario's `model = "../base-01/model.toml"`.
pub fn read_inside(root: &Path, base_relative: &str, relative: &str) -> Result<String, String> {
    let path = resolve_inside(root, base_relative, relative)?;
    let metadata = std::fs::metadata(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(format!("{} is not a bounded regular file", path.display()));
    }
    std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn read(root: &Path, relative: &str) -> Result<String, String> {
    let path = reject_symlinks(root, relative)?;
    let metadata = std::fs::metadata(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{} is not a regular file", path.display()));
    }
    if metadata.len() > MAX_FILE_BYTES {
        return Err(format!("{} is larger than the sandbox limit", path.display()));
    }
    std::fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn write(root: &Path, relative: &str, contents: &str) -> Result<String, String> {
    if contents.len() as u64 > MAX_FILE_BYTES {
        return Err("contents exceed the sandbox limit".into());
    }
    let path = reject_symlinks(root, relative)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    // Re-check after creating parents: a concurrent actor could have introduced a symlink.
    let path = reject_symlinks(root, relative)?;
    let parent = path.parent().ok_or_else(|| "sandbox path has no parent".to_string())?;
    let temporary = parent.join(format!(
        ".poieweave-write-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
    ));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        use std::io::Write;
        file.write_all(contents.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        std::fs::rename(&temporary, &path).map_err(|error| format!("{}: {error}", path.display()))?;
        Ok(path.display().to_string())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_resolve_under_the_root() {
        let root = Path::new("/tmp/poieweave-sandbox");
        assert_eq!(resolve(root, "notes/a.md").unwrap(), root.join("notes/a.md"));
        assert_eq!(resolve(root, "file.txt").unwrap(), root.join("file.txt"));
    }

    #[test]
    fn symlinks_are_rejected_for_read_and_write() {
        let root = std::env::temp_dir().join(format!("poieweave-sandbox-test-{}", std::process::id()));
        let outside = std::env::temp_dir().join(format!("poieweave-sandbox-outside-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&outside);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&outside, "secret").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, root.join("escape")).unwrap();
        #[cfg(unix)]
        {
            assert!(read(&root, "escape").is_err());
            assert!(write(&root, "escape", "nope").is_err());
        }
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&outside);
    }

    #[test]
    fn traversal_and_absolute_paths_are_rejected() {
        let root = Path::new("/tmp/poieweave-sandbox");
        for bad in ["", " ", "../secret", "notes/../../secret", "/etc/passwd", "./notes", "notes/../x"] {
            assert!(resolve(root, bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn read_inside_allows_sibling_but_not_escape() {
        let root = std::env::temp_dir().join(format!("poieweave-sandbox-inside-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("scenario")).unwrap();
        std::fs::create_dir_all(root.join("model")).unwrap();
        std::fs::write(root.join("model/model.toml"), "x").unwrap();
        std::fs::write(root.join("scenario/scenario.toml"), "y").unwrap();
        assert_eq!(read_inside(&root, "scenario", "../model/model.toml").unwrap(), "x");
        assert!(read_inside(&root, "scenario", "../../etc/passwd").is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}

use std::fs;
use std::path::{Path, PathBuf};

pub struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    pub fn new(root: impl AsRef<Path>) -> std::io::Result<Self> {
        let root = fs::canonicalize(root)?;
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn validate_path(&self, rel_path: impl AsRef<Path>) -> Result<PathBuf, String> {
        let joined = self.root.join(rel_path);
        
        // Check for traversal components before canonicalization if possible, or canonicalize parent
        let path_str = joined.to_string_lossy();
        if path_str.contains("..") {
            return Err("Security violation: Path escapes sandbox via traversal".into());
        }

        // If path exists or we can canonicalize normalized components
        let canonical = match fs::canonicalize(&joined) {
            Ok(p) => p,
            Err(_) => {
                // For non-existent files, check parent canonical path
                if let Some(parent) = joined.parent() {
                    let canon_parent = fs::canonicalize(parent)
                        .map_err(|_| "Security violation: Invalid base path".to_string())?;
                    if !canon_parent.starts_with(&self.root) {
                        return Err("Security violation: Path escapes sandbox".into());
                    }
                    joined
                } else {
                    return Err("Security violation: Invalid path".into());
                }
            }
        };

        if !canonical.starts_with(&self.root) {
            return Err("Security violation: Path escapes sandbox".into());
        }

        Ok(canonical)
    }

    pub fn atomic_write(&self, rel_path: impl AsRef<Path>, content: &str) -> Result<PathBuf, String> {
        let target = self.validate_path(rel_path)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let tmp_path = target.with_extension("pir.tmp");
        fs::write(&tmp_path, content).map_err(|e| e.to_string())?;
        fs::rename(&tmp_path, &target).map_err(|e| e.to_string())?;
        Ok(target)
    }
}

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TempDir(PathBuf);

impl TempDir {
    #[must_use]
    pub fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("mt-tabfinder-fs-{}-{name}", std::process::id()));
        drop(fs::remove_dir_all(&path));
        Self(path)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

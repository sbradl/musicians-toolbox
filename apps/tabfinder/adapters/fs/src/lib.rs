use std::fs::{self, ReadDir};
use std::path::PathBuf;

use mt_tabfinder_application::{
    FolderFile, LibraryFolder, LibraryFolderGone, RememberedLibraryLocation,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct FsLibraryFolder;

impl LibraryFolder for FsLibraryFolder {
    fn files(&self, location: &str) -> Result<Vec<FolderFile>, LibraryFolderGone> {
        let entries = fs::read_dir(location).map_err(|_| LibraryFolderGone)?;
        let mut files = Vec::new();
        collect_files(entries, &mut files);
        Ok(files)
    }
}

fn collect_files(entries: ReadDir, files: &mut Vec<FolderFile>) {
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_dir() {
            if let (false, Ok(subfolder)) = (file_name.starts_with('.'), fs::read_dir(entry.path()))
            {
                collect_files(subfolder, files);
            }
        } else if file_type.is_file() {
            files.push(FolderFile {
                id: entry.path().to_string_lossy().into_owned(),
                file_name,
            });
        }
    }
}

#[derive(Clone, Debug)]
pub struct FsRememberedLibraryLocation {
    file: PathBuf,
}

impl FsRememberedLibraryLocation {
    #[must_use]
    pub fn in_config_dir(config_dir: impl Into<PathBuf>) -> Self {
        Self {
            file: config_dir.into().join("library-location"),
        }
    }
}

// Failing to remember or forget only costs a re-pick or a failed Scan on the next start.
impl RememberedLibraryLocation for FsRememberedLibraryLocation {
    fn remembered(&self) -> Option<String> {
        let contents = fs::read_to_string(&self.file).ok()?;
        let location = contents.trim_end_matches(['\n', '\r']);
        (!location.is_empty()).then(|| location.to_owned())
    }

    fn remember(&self, location: &str) {
        if let Some(config_dir) = self.file.parent() {
            drop(fs::create_dir_all(config_dir));
        }
        drop(fs::write(&self.file, location));
    }

    fn forget(&self) {
        drop(fs::remove_file(&self.file));
    }
}

pub fn desktop_config_dir(env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    env("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|xdg| xdg.is_absolute())
        .or_else(|| env("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|config_home| config_home.join("tabfinder"))
}

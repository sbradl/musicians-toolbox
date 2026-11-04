use std::fs::{self, ReadDir};

use mt_tabfinder_application::{FolderFile, LibraryFolder, LibraryFolderGone};

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

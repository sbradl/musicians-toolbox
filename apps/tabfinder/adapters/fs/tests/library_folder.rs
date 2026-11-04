use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mt_tabfinder_application::{FolderFile, LibraryFolder, LibraryFolderGone};
use mt_tabfinder_fs::FsLibraryFolder;

struct TempFolder(PathBuf);

impl TempFolder {
    fn new(name: &str) -> io::Result<Self> {
        let path =
            std::env::temp_dir().join(format!("mt-tabfinder-fs-{}-{name}", std::process::id()));
        drop(fs::remove_dir_all(&path));
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn with_file(self, relative: &str) -> io::Result<Self> {
        let file = self.0.join(relative);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file, b"")?;
        Ok(self)
    }

    fn location(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }

    fn file(&self, relative: &str) -> FolderFile {
        FolderFile {
            id: self.0.join(relative).to_string_lossy().into_owned(),
            file_name: Path::new(relative)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
        }
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

fn sorted(mut files: Vec<FolderFile>) -> Vec<FolderFile> {
    files.sort_by(|a, b| a.id.cmp(&b.id));
    files
}

#[test]
fn lists_every_file_in_subfolders_but_not_in_hidden_folders() -> io::Result<()> {
    let folder = TempFolder::new("recursive")?
        .with_file("Angie.gp4")?
        .with_file("Beatles/1968/Blackbird.GP5")?
        .with_file("notes.txt")?
        .with_file(".backup/Angie.gp4")?
        .with_file("Beatles/.cache/Blackbird.gp5")?;

    let files = FsLibraryFolder.files(&folder.location());

    assert_eq!(
        files.map(sorted),
        Ok(sorted(vec![
            folder.file("Angie.gp4"),
            folder.file("Beatles/1968/Blackbird.GP5"),
            folder.file("notes.txt"),
        ]))
    );
    Ok(())
}

#[test]
fn a_folder_that_is_gone_is_reported() -> io::Result<()> {
    let folder = TempFolder::new("gone")?;
    let location = folder.location();
    drop(folder);

    assert_eq!(FsLibraryFolder.files(&location), Err(LibraryFolderGone));
    Ok(())
}

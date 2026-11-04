use mt_tab_library::Library;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderFile {
    pub id: String,
    pub file_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibraryFolderGone;

pub trait LibraryFolder: Send + Sync {
    fn files(&self, location: &str) -> Result<Vec<FolderFile>, LibraryFolderGone>;
}

pub fn scan(folder: &dyn LibraryFolder, location: &str) -> Result<Library, LibraryFolderGone> {
    let files = folder.files(location)?;
    Ok(Library::scanned(
        files.into_iter().map(|file| (file.id, file.file_name)),
    ))
}

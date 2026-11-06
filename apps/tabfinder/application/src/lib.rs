use std::fmt::Debug;

use mt_library::Library;
pub use mt_library::Song;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryLocation(String);

impl LibraryLocation {
    pub fn new(location: &str) -> Self {
        Self(location.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LibraryFolderGone;

pub trait LibraryFolder: Send + Sync + Debug {
    fn songs(&self, location: &LibraryLocation) -> Result<Vec<Song>, LibraryFolderGone>;
}

pub trait RememberedLibraryLocation: Send + Sync + Debug {
    fn remembered(&self) -> Option<LibraryLocation>;
    fn remember(&self, location: &LibraryLocation);
    fn forget(&self);
}

#[derive(Debug)]
pub struct Libraries {
    folder: Box<dyn LibraryFolder>,
    remembered: Box<dyn RememberedLibraryLocation>,
}

impl Libraries {
    pub fn new(
        folder: Box<dyn LibraryFolder>,
        remembered: Box<dyn RememberedLibraryLocation>,
    ) -> Self {
        Self { folder, remembered }
    }

    pub fn remembered_location(&self) -> Option<LibraryLocation> {
        self.remembered.remembered()
    }

    pub fn open(&self, location: &LibraryLocation) -> Result<Library, LibraryFolderGone> {
        let library = self.scan(location);

        if library.is_ok() {
            self.remembered.remember(location);
        } else if self.is_remembered(location) {
            self.remembered.forget();
        }

        library
    }

    fn scan(&self, location: &LibraryLocation) -> Result<Library, LibraryFolderGone> {
        let songs = self.folder.songs(location)?;

        Ok(Library::of(songs))
    }

    fn is_remembered(&self, location: &LibraryLocation) -> bool {
        let Some(remembered) = self.remembered.remembered() else {
            return false;
        };

        &remembered == location
    }
}

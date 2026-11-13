use std::fmt::Debug;

use mt_library::Library;
pub use mt_library::Song;
pub use mt_music_kernel::{Pitch, Tuning};
pub use mt_score::LARGEST_TAB_FILE;
use mt_score::{ScoreHeader, read_header};

pub fn song_read_from_tab_file(mut song: Song, tab_file: Option<&[u8]>) -> Song {
    let Some(tab_file) = tab_file else {
        return unreadable(song);
    };
    let Ok(header) = read_header(tab_file) else {
        return unreadable(song);
    };
    song.tunings = distinct_tunings_of(header);

    song
}

fn distinct_tunings_of(header: ScoreHeader) -> Vec<Tuning> {
    let mut tunings = Vec::new();
    for track in header.tracks {
        let Some(tuning) = track.tuning else {
            continue;
        };

        if !tunings.contains(&tuning) {
            tunings.push(tuning);
        }
    }

    tunings
}

const fn unreadable(mut song: Song) -> Song {
    song.is_unreadable = true;

    song
}

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
pub enum LibraryFolderError {
    Gone,
    NoAccess,
}

pub trait LibraryFolder: Send + Sync + Debug {
    fn songs(&self, location: &LibraryLocation) -> Result<Vec<Song>, LibraryFolderError>;
}

pub trait RememberedLibraryLocation: Send + Sync + Debug {
    fn remembered(&self) -> Option<LibraryLocation>;
    fn remember(&self, location: &LibraryLocation);
    fn forget(&self);
}

pub trait LastScan: Send + Sync + Debug {
    fn songs_of(&self, location: &LibraryLocation) -> Option<Vec<Song>>;
    fn keep(&self, location: &LibraryLocation, songs: &[Song]);
}

#[derive(Debug)]
pub struct Libraries {
    folder: Box<dyn LibraryFolder>,
    remembered: Box<dyn RememberedLibraryLocation>,
    last_scan: Box<dyn LastScan>,
}

impl Libraries {
    pub fn new(
        folder: Box<dyn LibraryFolder>,
        remembered: Box<dyn RememberedLibraryLocation>,
        last_scan: Box<dyn LastScan>,
    ) -> Self {
        Self {
            folder,
            remembered,
            last_scan,
        }
    }

    pub fn last_scan(&self, location: &LibraryLocation) -> Option<Library> {
        let songs = self.last_scan.songs_of(location)?;

        Some(Library::of(songs))
    }

    pub fn remembered_location(&self) -> Option<LibraryLocation> {
        self.remembered.remembered()
    }

    pub fn open(&self, location: &LibraryLocation) -> Result<Library, LibraryFolderError> {
        let library = self.scan(location);

        if let Ok(scanned) = &library {
            self.remembered.remember(location);
            self.last_scan.keep(location, scanned.songs());
        } else if self.is_remembered(location) {
            self.remembered.forget();
        }

        library
    }

    fn scan(&self, location: &LibraryLocation) -> Result<Library, LibraryFolderError> {
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

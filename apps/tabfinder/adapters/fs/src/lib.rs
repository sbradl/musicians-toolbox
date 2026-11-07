use std::fs::{self, DirEntry, ReadDir};
use std::io;
use std::path::{Path, PathBuf};

use mt_tabfinder_application::{
    LibraryFolder, LibraryFolderError, LibraryLocation, RememberedLibraryLocation, Song,
};

#[derive(Debug)]
pub struct FsLibraryFolder;

impl LibraryFolder for FsLibraryFolder {
    fn songs(&self, location: &LibraryLocation) -> Result<Vec<Song>, LibraryFolderError> {
        let folder = open_library_folder(Path::new(location.as_str()))?;

        Ok(songs_in(folder))
    }
}

fn open_library_folder(folder: &Path) -> Result<ReadDir, LibraryFolderError> {
    match fs::read_dir(folder) {
        Ok(entries) => Ok(entries),
        Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
            Err(LibraryFolderError::NoAccess)
        }
        Err(_) => Err(LibraryFolderError::Gone),
    }
}

fn songs_in(folder: ReadDir) -> Vec<Song> {
    let mut songs = Vec::new();
    for entry in folder {
        let Ok(entry) = entry else {
            continue;
        };

        if is_visible_subfolder(&entry) {
            songs.extend(songs_in_subfolder(&entry));
        } else if let Some(song) = song_from(&entry) {
            songs.push(song);
        }
    }

    songs
}

fn songs_in_subfolder(subfolder: &DirEntry) -> Vec<Song> {
    let Some(folder) = open_folder(&subfolder.path()) else {
        return Vec::new();
    };

    songs_in(folder)
}

fn song_from(entry: &DirEntry) -> Option<Song> {
    if !is_file(entry) {
        return None;
    }

    song_from_file_name(&entry.file_name().to_string_lossy())
}

pub fn song_from_file_name(file_name: &str) -> Option<Song> {
    let file = Path::new(file_name);

    if file_name.starts_with('.') || !is_tab_file(file) {
        return None;
    }

    Some(Song {
        title: file_name_without_extension(file)?,
    })
}

fn open_folder(folder: &Path) -> Option<ReadDir> {
    fs::read_dir(folder).ok()
}

fn is_visible_subfolder(entry: &DirEntry) -> bool {
    is_folder(entry) && !is_hidden(entry)
}

fn is_folder(entry: &DirEntry) -> bool {
    let Ok(kind) = entry.file_type() else {
        return false;
    };

    kind.is_dir()
}

fn is_file(entry: &DirEntry) -> bool {
    let Ok(kind) = entry.file_type() else {
        return false;
    };

    kind.is_file()
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name().to_string_lossy().starts_with('.')
}

fn is_tab_file(file: &Path) -> bool {
    let Some(extension) = file.extension() else {
        return false;
    };
    let extension = extension.to_string_lossy().to_lowercase();
    matches!(
        extension.as_str(),
        "gp3" | "gp4" | "gp5" | "gpx" | "gp" | "tg" | "ptb"
    )
}

fn file_name_without_extension(file: &Path) -> Option<String> {
    Some(file.file_stem()?.to_string_lossy().into_owned())
}

#[derive(Debug)]
pub struct FsRememberedLibraryLocation {
    file: PathBuf,
}

impl FsRememberedLibraryLocation {
    pub fn in_config_dir(config_dir: impl Into<PathBuf>) -> Self {
        Self {
            file: config_dir.into().join("library-location"),
        }
    }
}

impl RememberedLibraryLocation for FsRememberedLibraryLocation {
    fn remembered(&self) -> Option<LibraryLocation> {
        let contents = fs::read_to_string(&self.file).ok()?;
        let location = contents.trim_end_matches(['\n', '\r']);

        if location.is_empty() {
            None
        } else {
            Some(LibraryLocation::new(location))
        }
    }

    fn remember(&self, location: &LibraryLocation) {
        ignore_failure(self.write(location));
    }

    fn forget(&self) {
        ignore_failure(fs::remove_file(&self.file));
    }
}

impl FsRememberedLibraryLocation {
    fn write(&self, location: &LibraryLocation) -> io::Result<()> {
        if let Some(config_dir) = self.file.parent() {
            fs::create_dir_all(config_dir)?;
        }

        fs::write(&self.file, location.as_str())
    }
}

// A failure only costs the musician a re-pick on the next start.
fn ignore_failure(result: io::Result<()>) {
    drop(result);
}

pub fn desktop_config_dir(variable: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let config_home = match variable("XDG_CONFIG_HOME").map(PathBuf::from) {
        Some(xdg_config_home) if xdg_config_home.is_absolute() => xdg_config_home,
        _ => PathBuf::from(variable("HOME")?).join(".config"),
    };

    Some(config_home.join("tabfinder"))
}

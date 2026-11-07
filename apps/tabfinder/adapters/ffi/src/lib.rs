use std::fmt;

use mt_tabfinder_application::{self as application, LibraryLocation, Song};
use mt_tabfinder_fs::{FsLibraryFolder, FsRememberedLibraryLocation};
use mt_tabfinder_presentation as presentation;
use presentation::{Action, AppViewModel, Effect, LibraryViewModel, SongItem};

uniffi::setup_scaffolding!();

#[uniffi::remote(Record)]
pub struct AppViewModel {
    pub library: LibraryViewModel,
}

#[uniffi::remote(Record)]
pub struct LibraryViewModel {
    pub status: String,
    pub problem: Option<String>,
    pub choose_folder: Action,
    pub songs: Vec<SongItem>,
}

#[uniffi::remote(Record)]
pub struct SongItem {
    pub title: String,
}

#[uniffi::remote(Record)]
pub struct Action {
    pub label: String,
    pub is_enabled: bool,
}

#[uniffi::remote(Record)]
pub struct Song {
    pub title: String,
}

#[uniffi::remote(Enum)]
pub enum Effect {
    PickFolder { title: String },
}

#[uniffi::export(callback_interface)]
pub trait ViewModelListener: Send + Sync {
    fn on_view_model(&self, view_model: AppViewModel);
    fn on_effect(&self, effect: Effect);
}

struct ListenerInKotlin(Box<dyn ViewModelListener>);

impl presentation::ViewModelListener for ListenerInKotlin {
    fn on_view_model(&self, view_model: AppViewModel) {
        self.0.on_view_model(view_model);
    }

    fn on_effect(&self, effect: Effect) {
        self.0.on_effect(effect);
    }
}

#[derive(uniffi::Error, Debug)]
pub enum LibraryFolderError {
    Gone,
    NoAccess,
}

#[expect(
    clippy::panic,
    clippy::fallible_impl_from,
    reason = "an unexpected Kotlin exception is a bug, and bugs fail fast (ADR 0002); user decision"
)]
impl From<uniffi::UnexpectedUniFFICallbackError> for LibraryFolderError {
    fn from(error: uniffi::UnexpectedUniFFICallbackError) -> Self {
        panic!("The Library folder in Kotlin failed unexpectedly: {error}")
    }
}

#[uniffi::export(callback_interface)]
pub trait LibraryFolder: Send + Sync {
    fn songs(&self, location: String) -> Result<Vec<Song>, LibraryFolderError>;
}

struct LibraryFolderInKotlin(Box<dyn LibraryFolder>);

impl fmt::Debug for LibraryFolderInKotlin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LibraryFolderInKotlin")
            .finish_non_exhaustive()
    }
}

impl application::LibraryFolder for LibraryFolderInKotlin {
    fn songs(
        &self,
        location: &LibraryLocation,
    ) -> Result<Vec<Song>, application::LibraryFolderError> {
        match self.0.songs(location.as_str().to_owned()) {
            Ok(songs) => Ok(songs),
            Err(LibraryFolderError::Gone) => Err(application::LibraryFolderError::Gone),
            Err(LibraryFolderError::NoAccess) => Err(application::LibraryFolderError::NoAccess),
        }
    }
}

#[uniffi::export]
pub fn song_from_file_name(file_name: &str) -> Option<Song> {
    mt_tabfinder_fs::song_from_file_name(file_name)
}

#[uniffi::export]
pub fn desktop_config_dir() -> Option<String> {
    let config_dir = mt_tabfinder_fs::desktop_config_dir(environment_variable)?;

    config_dir.into_os_string().into_string().ok()
}

fn environment_variable(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

#[derive(uniffi::Object, Debug)]
pub struct App(presentation::App);

#[uniffi::export]
impl App {
    #[uniffi::constructor]
    pub fn new(listener: Box<dyn ViewModelListener>, config_dir: &str) -> Self {
        Self::wired(listener, config_dir, Box::new(FsLibraryFolder))
    }

    #[uniffi::constructor]
    pub fn with_library_folder(
        listener: Box<dyn ViewModelListener>,
        config_dir: &str,
        library_folder: Box<dyn LibraryFolder>,
    ) -> Self {
        Self::wired(
            listener,
            config_dir,
            Box::new(LibraryFolderInKotlin(library_folder)),
        )
    }

    pub fn start(&self) {
        self.0.start();
    }

    pub fn choose_folder(&self) {
        self.0.choose_folder();
    }

    pub fn folder_picked(&self, location: &str) {
        self.0.folder_picked(location);
    }
}

impl App {
    fn wired(
        listener: Box<dyn ViewModelListener>,
        config_dir: &str,
        library_folder: Box<dyn application::LibraryFolder>,
    ) -> Self {
        Self(presentation::App::new(
            Box::new(ListenerInKotlin(listener)),
            library_folder,
            Box::new(FsRememberedLibraryLocation::in_config_dir(config_dir)),
        ))
    }
}

use std::fmt;

use mt_tabfinder_fs::{FsLibraryFolder, FsRememberedLibraryLocation};
use mt_tabfinder_presentation as presentation;
use presentation::{Action, AppViewModel, Effect, LibraryViewModel, SongRow};

uniffi::setup_scaffolding!();

#[uniffi::remote(Record)]
pub struct AppViewModel {
    pub library: LibraryViewModel,
}

#[uniffi::remote(Record)]
pub struct LibraryViewModel {
    pub status: String,
    pub choose_folder: Action,
    pub songs: Vec<SongRow>,
}

#[uniffi::remote(Record)]
pub struct SongRow {
    pub id: String,
    pub title: String,
}

#[uniffi::remote(Record)]
pub struct Action {
    pub label: String,
    pub is_enabled: bool,
}

#[uniffi::remote(Enum)]
pub enum Effect {
    PickFolder,
}

#[uniffi::export(callback_interface)]
pub trait ViewModelListener: Send + Sync {
    fn on_view_model(&self, view_model: AppViewModel);
    fn on_effect(&self, effect: Effect);
}

struct ForeignViewModelListener(Box<dyn ViewModelListener>);

impl presentation::ViewModelListener for ForeignViewModelListener {
    fn on_view_model(&self, view_model: AppViewModel) {
        self.0.on_view_model(view_model);
    }

    fn on_effect(&self, effect: Effect) {
        self.0.on_effect(effect);
    }
}

#[uniffi::export]
#[must_use]
pub fn desktop_config_dir() -> Option<String> {
    mt_tabfinder_fs::desktop_config_dir(|name| std::env::var(name).ok())
        .and_then(|dir| dir.into_os_string().into_string().ok())
}

#[derive(uniffi::Object)]
pub struct App(presentation::App);

impl fmt::Debug for App {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("App").field(&self.0).finish()
    }
}

#[uniffi::export]
impl App {
    #[uniffi::constructor]
    #[must_use]
    pub fn new(listener: Box<dyn ViewModelListener>, config_dir: &str) -> Self {
        Self(presentation::App::new(
            Box::new(ForeignViewModelListener(listener)),
            Box::new(FsLibraryFolder),
            Box::new(FsRememberedLibraryLocation::in_config_dir(config_dir)),
        ))
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

    pub const fn folder_pick_cancelled(&self) {
        self.0.folder_pick_cancelled();
    }
}

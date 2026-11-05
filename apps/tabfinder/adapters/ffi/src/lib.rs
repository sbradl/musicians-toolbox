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

#[uniffi::remote(Enum)]
pub enum Effect {
    PickFolder,
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
        Self(presentation::App::new(
            Box::new(ListenerInKotlin(listener)),
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
}

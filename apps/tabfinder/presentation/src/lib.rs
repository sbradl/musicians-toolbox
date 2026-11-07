mod library_screen;
mod texts;

use std::fmt;

use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use mt_tabfinder_application::{
    Libraries, LibraryFolder, LibraryFolderGone, LibraryLocation, RememberedLibraryLocation,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppViewModel {
    pub library: LibraryViewModel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryViewModel {
    pub status: String,
    pub choose_folder: Action,
    pub songs: Vec<SongItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SongItem {
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub label: String,
    pub is_enabled: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    PickFolder { title: String },
}

pub trait ViewModelListener: Send + Sync {
    fn on_view_model(&self, view_model: AppViewModel);
    fn on_effect(&self, effect: Effect);
}

pub struct App {
    listener: Box<dyn ViewModelListener>,
    libraries: Libraries,
    texts: Texts,
}

impl fmt::Debug for App {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("App").finish_non_exhaustive()
    }
}

impl App {
    pub fn new(
        listener: Box<dyn ViewModelListener>,
        library_folder: Box<dyn LibraryFolder>,
        remembered_library_location: Box<dyn RememberedLibraryLocation>,
    ) -> Self {
        Self {
            listener,
            libraries: Libraries::new(library_folder, remembered_library_location),
            texts: Texts::english(),
        }
    }

    pub fn start(&self) {
        match self.libraries.remembered_location() {
            Some(location) => self.open_library(&location),
            None => self.show(LibraryViewModel::no_folder_chosen(&self.texts)),
        }
    }

    pub fn choose_folder(&self) {
        self.listener.on_effect(Effect::PickFolder {
            title: self.texts.library_folder_dialog_title(),
        });
    }

    pub fn folder_picked(&self, location: &str) {
        self.open_library(&LibraryLocation::new(location));
    }

    fn open_library(&self, location: &LibraryLocation) {
        self.show(LibraryViewModel::scanning(&self.texts));
        let library = match self.libraries.open(location) {
            Ok(library) => LibraryViewModel::listing(&library, &self.texts),
            Err(LibraryFolderGone) => LibraryViewModel::no_folder_chosen(&self.texts),
        };
        self.show(library);
    }

    fn show(&self, library: LibraryViewModel) {
        self.listener.on_view_model(AppViewModel { library });
    }
}

struct Texts(FluentBundle<FluentResource>);

mod library_screen;
mod texts;

use std::fmt;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use mt_tabfinder_application::{
    Libraries, LibraryFolder, LibraryLocation, RememberedLibraryLocation,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppViewModel {
    pub library: LibraryViewModel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryViewModel {
    pub status: String,
    pub problem: Option<String>,
    pub choose_folder: Action,
    pub rescan: Action,
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
    ShowMessage { text: String },
}

pub trait ViewModelListener: Send + Sync {
    fn on_view_model(&self, view_model: AppViewModel);
    fn on_effect(&self, effect: Effect);
}

pub struct App {
    listener: Box<dyn ViewModelListener>,
    libraries: Libraries,
    texts: Texts,
    shown: Mutex<Option<LibraryViewModel>>,
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
            shown: Mutex::new(None),
        }
    }

    pub fn start(&self) {
        match self.libraries.remembered_location() {
            Some(location) => _ = self.open_library(&location),
            None => self.show(LibraryViewModel::no_folder_chosen(&self.texts)),
        }
    }

    pub fn choose_folder(&self) {
        self.listener.on_effect(Effect::PickFolder {
            title: self.texts.library_folder_dialog_title(),
        });
    }

    pub fn folder_picked(&self, location: &str) {
        _ = self.open_library(&LibraryLocation::new(location));
    }

    pub fn rescan(&self) {
        let Some(location) = self.libraries.remembered_location() else {
            return;
        };
        let Some(song_count) = self.open_library(&location) else {
            return;
        };
        let text = self
            .texts
            .rescan_finished(song_count, UNREADABLE_WITHOUT_TAB_FILE_READERS);

        self.listener.on_effect(Effect::ShowMessage { text });
    }

    pub fn folder_picker_unavailable(&self) {
        let shown = self.shown_library();
        self.show(shown.because(self.texts.folder_picker_unavailable()));
    }

    fn open_library(&self, location: &LibraryLocation) -> Option<usize> {
        self.show(LibraryViewModel::scanning(&self.texts));
        match self.libraries.open(location) {
            Ok(library) => {
                self.show(LibraryViewModel::listing(&library, &self.texts));

                Some(library.songs().len())
            }
            Err(error) => {
                self.show(
                    LibraryViewModel::no_folder_chosen(&self.texts)
                        .because(self.texts.library_folder_problem(&error)),
                );

                None
            }
        }
    }

    fn show(&self, library: LibraryViewModel) {
        *lock(&self.shown) = Some(library.clone());
        self.listener.on_view_model(AppViewModel { library });
    }

    fn shown_library(&self) -> LibraryViewModel {
        let shown = lock(&self.shown).clone();

        shown.unwrap_or_else(|| LibraryViewModel::no_folder_chosen(&self.texts))
    }
}

const UNREADABLE_WITHOUT_TAB_FILE_READERS: usize = 0;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

struct Texts(FluentBundle<FluentResource>);

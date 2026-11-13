mod library_screen;
mod texts;

use std::fmt;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use mt_library::{Filter, Library};
use mt_tabfinder_application::{
    LastScan, Libraries, LibraryFolder, LibraryLocation, RememberedLibraryLocation,
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
    pub filters: Option<Filters>,
    pub songs: Vec<SongItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filters {
    pub artist: FilterField,
    pub song: FilterField,
    pub match_count: String,
    pub nothing_matches: Option<NothingMatches>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterField {
    pub label: String,
    pub text: String,
    pub clear: Action,
    pub suggestions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NothingMatches {
    pub text: String,
    pub clear_filters: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SongItem {
    pub title: String,
    pub artist_and_album: Option<String>,
    pub tunings: Vec<String>,
    pub unreadable: Option<String>,
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
    library: Mutex<Option<ListedLibrary>>,
    filter: Mutex<Filter>,
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
        last_scan: Box<dyn LastScan>,
    ) -> Self {
        Self {
            listener,
            libraries: Libraries::new(library_folder, remembered_library_location, last_scan),
            texts: Texts::english(),
            shown: Mutex::new(None),
            library: Mutex::new(None),
            filter: Mutex::new(Filter::default()),
        }
    }

    pub fn start(&self) {
        let Some(location) = self.libraries.remembered_location() else {
            self.show(LibraryViewModel::no_folder_chosen(&self.texts));

            return;
        };

        match self.libraries.last_scan(&location) {
            Some(library) => _ = self.list(library),
            None => _ = self.open_library(&location),
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
        let Some(scanned) = self.open_library(&location) else {
            return;
        };
        let text = self
            .texts
            .rescan_finished(scanned.songs, scanned.unreadable);

        self.listener.on_effect(Effect::ShowMessage { text });
    }

    pub fn song_filter_changed(&self, text: &str) {
        lock(&self.filter).set_song(text);
        self.show_library();
    }

    pub fn artist_filter_changed(&self, text: &str) {
        lock(&self.filter).set_artist(text);
        self.show_library();
    }

    pub fn clear_filters(&self) {
        *lock(&self.filter) = Filter::default();
        self.show_library();
    }

    pub fn folder_picker_unavailable(&self) {
        let shown = self.shown_library();
        self.show(shown.because(self.texts.folder_picker_unavailable()));
    }

    fn open_library(&self, location: &LibraryLocation) -> Option<Scanned> {
        self.show(LibraryViewModel::scanning(&self.texts));
        match self.libraries.open(location) {
            Ok(library) => Some(self.list(library)),
            Err(error) => {
                *lock(&self.library) = None;
                self.show(
                    LibraryViewModel::no_folder_chosen(&self.texts)
                        .because(self.texts.library_folder_problem(&error)),
                );

                None
            }
        }
    }

    fn list(&self, library: Library) -> Scanned {
        let scanned = Scanned {
            songs: library.songs().len(),
            unreadable: library.unreadable_songs(),
        };
        *lock(&self.library) = Some(ListedLibrary::of(library, &self.texts));
        self.show_library();

        scanned
    }

    fn show_library(&self) {
        let filter = lock(&self.filter).clone();
        let Some(listing) = self.listing(&filter) else {
            return;
        };
        self.show(listing);
    }

    fn listing(&self, filter: &Filter) -> Option<LibraryViewModel> {
        let listed = lock(&self.library);

        Some(LibraryViewModel::listing(
            listed.as_ref()?,
            filter,
            &self.texts,
        ))
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

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

struct Texts(FluentBundle<FluentResource>);

#[derive(Debug)]
struct Scanned {
    songs: usize,
    unreadable: usize,
}

#[derive(Debug)]
struct ListedLibrary {
    library: Library,
    items: Vec<SongItem>,
}

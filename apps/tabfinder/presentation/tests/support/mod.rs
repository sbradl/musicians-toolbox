use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fluent_bundle as _;
use mt_library as _;
use unic_langid as _;

use mt_tabfinder_application::{
    LibraryFolder, LibraryFolderError, LibraryLocation, RememberedLibraryLocation, Song,
};
use mt_tabfinder_presentation::{
    Action, App, AppViewModel, Effect, LibraryViewModel, SongItem, ViewModelListener,
};

#[derive(Debug, Default)]
pub struct TabFinderSetup {
    folders: FakeLibraryFolders,
    remembered: FakeRememberedLibraryLocation,
}

impl TabFinderSetup {
    pub fn with_folder(self, location: &str, song_titles: &[&str]) -> Self {
        self.folders.add(location, song_titles);

        self
    }

    pub fn with_folder_it_may_not_read(self, location: &str) -> Self {
        self.folders.lock_away(location);

        self
    }

    pub fn remembering(self, location: &str) -> Self {
        self.remembered.remember(&LibraryLocation::new(location));

        self
    }

    pub fn start(self) -> TabFinder {
        let screens = ShownScreens::default();
        let app = App::new(
            Box::new(screens.clone()),
            Box::new(self.folders.clone()),
            Box::new(self.remembered.clone()),
        );
        app.start();
        TabFinder {
            app,
            screens,
            folders: self.folders,
            remembered: self.remembered,
        }
    }
}

#[derive(Debug)]
pub struct TabFinder {
    app: App,
    screens: ShownScreens,
    folders: FakeLibraryFolders,
    remembered: FakeRememberedLibraryLocation,
}

impl TabFinder {
    pub fn without_folders() -> TabFinderSetup {
        TabFinderSetup::default()
    }

    pub fn with_folder(location: &str, song_titles: &[&str]) -> TabFinderSetup {
        TabFinderSetup::default().with_folder(location, song_titles)
    }

    pub fn with_folder_it_may_not_read(location: &str) -> TabFinderSetup {
        TabFinderSetup::default().with_folder_it_may_not_read(location)
    }

    pub fn musician_clicks_choose_folder(&self) {
        self.app.choose_folder();
    }

    pub fn musician_picks_folder(&self, location: &str) {
        self.app.choose_folder();
        self.app.folder_picked(location);
    }

    pub fn musician_finds_no_folder_dialog(&self) {
        self.app.choose_folder();
        self.app.folder_picker_unavailable();
    }

    pub fn musician_cancels_the_folder_pick(&self) {
        self.app.choose_folder();
    }

    pub fn folder_is_deleted(&self, location: &str) {
        self.folders.delete(location);
    }

    pub fn library_screens_shown(&self) -> Vec<LibraryViewModel> {
        lock(&self.screens.library).clone()
    }

    pub fn statuses_shown(&self) -> Vec<String> {
        let mut statuses = Vec::new();
        for screen in self.library_screens_shown() {
            statuses.push(screen.status);
        }

        statuses
    }

    #[track_caller]
    pub fn assert_library_screen(&self, expected: &LibraryViewModel) {
        assert_eq!(self.library_screens_shown().last(), Some(expected));
    }

    #[track_caller]
    pub fn assert_status(&self, expected: &str) {
        assert_eq!(self.statuses_shown().pop(), Some(expected.to_owned()));
    }

    #[track_caller]
    pub fn assert_song_titles(&self, expected: &[&str]) {
        assert_eq!(self.song_titles_shown(), expected);
    }

    fn song_titles_shown(&self) -> Vec<String> {
        let Some(last_screen) = self.library_screens_shown().pop() else {
            return Vec::new();
        };
        let mut titles = Vec::new();
        for song in last_screen.songs {
            titles.push(song.title);
        }

        titles
    }

    #[track_caller]
    pub fn assert_platform_was_asked_to_pick_a_folder_titled(&self, title: &str) {
        let pick_folder = Effect::PickFolder {
            title: title.to_owned(),
        };

        assert_eq!(*lock(&self.screens.effects), [pick_folder]);
    }

    #[track_caller]
    pub fn assert_remembers(&self, expected: &str) {
        assert_eq!(
            self.remembered.remembered(),
            Some(LibraryLocation::new(expected))
        );
    }

    #[track_caller]
    pub fn assert_remembers_nothing(&self) {
        assert_eq!(self.remembered.remembered(), None);
    }
}

pub fn no_folder_chosen() -> LibraryViewModel {
    listing("No folder chosen", &[])
}

pub fn no_folder_chosen_because(problem: &str) -> LibraryViewModel {
    because(no_folder_chosen(), problem)
}

pub fn because(mut screen: LibraryViewModel, problem: &str) -> LibraryViewModel {
    screen.problem = Some(problem.to_owned());

    screen
}

pub fn scanning() -> LibraryViewModel {
    let mut screen = listing("Scanning…", &[]);
    screen.choose_folder.is_enabled = false;

    screen
}

pub fn listing(status: &str, song_titles: &[&str]) -> LibraryViewModel {
    LibraryViewModel {
        status: status.to_owned(),
        problem: None,
        choose_folder: Action {
            label: "Choose folder".to_owned(),
            is_enabled: true,
        },
        songs: song_items(song_titles),
    }
}

fn song_items(titles: &[&str]) -> Vec<SongItem> {
    let mut items = Vec::new();
    for title in titles {
        items.push(SongItem {
            title: (*title).to_owned(),
        });
    }

    items
}

type Shared<T> = Arc<Mutex<T>>;

#[derive(Clone, Debug, Default)]
struct ShownScreens {
    library: Shared<Vec<LibraryViewModel>>,
    effects: Shared<Vec<Effect>>,
}

impl ViewModelListener for ShownScreens {
    fn on_view_model(&self, view_model: AppViewModel) {
        lock(&self.library).push(view_model.library);
    }

    fn on_effect(&self, effect: Effect) {
        lock(&self.effects).push(effect);
    }
}

#[derive(Clone, Debug, Default)]
struct FakeLibraryFolders {
    songs_by_location: Shared<HashMap<String, Vec<Song>>>,
    locked_away: Shared<Vec<String>>,
}

impl FakeLibraryFolders {
    fn add(&self, location: &str, song_titles: &[&str]) {
        let mut songs = Vec::new();
        for title in song_titles {
            songs.push(Song::titled(title));
        }
        drop(lock(&self.songs_by_location).insert(location.to_owned(), songs));
    }

    fn lock_away(&self, location: &str) {
        lock(&self.locked_away).push(location.to_owned());
    }

    fn delete(&self, location: &str) {
        drop(lock(&self.songs_by_location).remove(location));
    }
}

impl LibraryFolder for FakeLibraryFolders {
    fn songs(&self, location: &LibraryLocation) -> Result<Vec<Song>, LibraryFolderError> {
        if self.is_locked_away(location.as_str()) {
            return Err(LibraryFolderError::NoAccess);
        }

        self.songs_at(location.as_str())
            .ok_or(LibraryFolderError::Gone)
    }
}

impl FakeLibraryFolders {
    fn is_locked_away(&self, location: &str) -> bool {
        lock(&self.locked_away)
            .iter()
            .any(|locked| locked == location)
    }

    fn songs_at(&self, location: &str) -> Option<Vec<Song>> {
        lock(&self.songs_by_location).get(location).cloned()
    }
}

#[derive(Clone, Debug, Default)]
struct FakeRememberedLibraryLocation {
    location: Shared<Option<LibraryLocation>>,
}

impl RememberedLibraryLocation for FakeRememberedLibraryLocation {
    fn remembered(&self) -> Option<LibraryLocation> {
        lock(&self.location).clone()
    }

    fn remember(&self, location: &LibraryLocation) {
        *lock(&self.location) = Some(location.clone());
    }

    fn forget(&self) {
        *lock(&self.location) = None;
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

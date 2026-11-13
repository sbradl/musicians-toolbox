use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use fluent_bundle as _;
use mt_library as _;
use unic_langid as _;

use mt_tabfinder_application::{
    LibraryFolder, LibraryFolderError, LibraryLocation, RememberedLibraryLocation, Song,
};
use mt_tabfinder_presentation::{
    Action, App, AppViewModel, Effect, FilterField, Filters, LibraryViewModel, NothingMatches,
    SongItem, ViewModelListener,
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

    pub fn with_folder_of_songs(self, location: &str, songs: &[SongInFolder]) -> Self {
        let mut found = Vec::new();
        for song in songs {
            found.push(song.0.clone());
        }
        self.folders.hold(location, found);

        self
    }

    pub fn with_ten_thousand_songs_in(self, location: &str) -> Self {
        let mut songs = Vec::new();
        for number in 0..10_000 {
            songs.push(
                song(&format!("Wexmoor {number}"))
                    .by("Corvid Lane")
                    .on("Saltglass"),
            );
        }

        self.with_folder_of_songs(location, &songs)
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

    pub fn with_folder_of_songs(location: &str, songs: &[SongInFolder]) -> TabFinderSetup {
        TabFinderSetup::default().with_folder_of_songs(location, songs)
    }

    pub fn with_ten_thousand_songs_in(location: &str) -> TabFinderSetup {
        TabFinderSetup::default().with_ten_thousand_songs_in(location)
    }

    pub fn time_for(&self, action: impl FnOnce()) -> Duration {
        let started = Instant::now();
        action();

        started.elapsed()
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

    pub fn musician_clicks_rescan(&self) {
        self.app.rescan();
    }

    pub fn musician_types_in_the_song_field(&self, text: &str) {
        self.app.song_filter_changed(text);
    }

    pub fn musician_types_in_the_artist_field(&self, text: &str) {
        self.app.artist_filter_changed(text);
    }

    pub fn musician_clicks_clear_filters(&self) {
        self.app.clear_filters();
    }

    pub fn folder_now_holds(&self, location: &str, song_titles: &[&str]) {
        self.folders.add(location, song_titles);
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
    pub fn assert_rescan_offered(&self) {
        assert_eq!(self.rescan_offered(), Some(true));
    }

    #[track_caller]
    pub fn assert_rescan_not_offered(&self) {
        assert_eq!(self.rescan_offered(), Some(false));
    }

    fn rescan_offered(&self) -> Option<bool> {
        let last_screen = self.library_screens_shown().pop()?;

        Some(last_screen.rescan.is_enabled)
    }

    #[track_caller]
    pub fn assert_match_count(&self, expected: &str) {
        assert_eq!(
            self.filters_shown().map(|filters| filters.match_count),
            Some(expected.to_owned())
        );
    }

    #[track_caller]
    pub fn assert_nothing_matches(&self, text: &str, clear_filters: &str) {
        let nothing_matches = NothingMatches {
            text: text.to_owned(),
            clear_filters: clear_filters.to_owned(),
        };

        assert_eq!(self.nothing_matches_shown(), Some(nothing_matches));
    }

    #[track_caller]
    pub fn assert_something_matches(&self) {
        assert_eq!(self.nothing_matches_shown(), None);
    }

    fn nothing_matches_shown(&self) -> Option<NothingMatches> {
        self.filters_shown()?.nothing_matches
    }

    #[track_caller]
    pub fn assert_song_field_shows(&self, expected: &str) {
        let song_field = self.filters_shown().map(|filters| filters.song.text);

        assert_eq!(song_field, Some(expected.to_owned()));
    }

    #[track_caller]
    pub fn assert_artist_field(&self, expected: &FilterField) {
        let artist_field = self.filters_shown().map(|filters| filters.artist);

        assert_eq!(artist_field.as_ref(), Some(expected));
    }

    #[track_caller]
    pub fn assert_artist_suggestions(&self, expected: &[&str]) {
        let suggestions = self
            .filters_shown()
            .map(|filters| filters.artist.suggestions);

        assert_eq!(suggestions, Some(owned_strings(expected)));
    }

    fn filters_shown(&self) -> Option<Filters> {
        self.library_screens_shown().pop()?.filters
    }

    #[track_caller]
    pub fn assert_song_titles(&self, expected: &[&str]) {
        assert_eq!(self.song_titles_shown(), expected);
    }

    #[track_caller]
    pub fn assert_song_rows(&self, expected: &[SongItem]) {
        assert_eq!(self.song_rows_shown(), expected);
    }

    fn song_rows_shown(&self) -> Vec<SongItem> {
        let Some(last_screen) = self.library_screens_shown().pop() else {
            return Vec::new();
        };

        last_screen.songs
    }

    fn song_titles_shown(&self) -> Vec<String> {
        let mut titles = Vec::new();
        for song in self.song_rows_shown() {
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
    pub fn assert_message_shown(&self, text: &str) {
        let message = Effect::ShowMessage {
            text: text.to_owned(),
        };

        assert!(lock(&self.screens.effects).contains(&message));
    }

    #[track_caller]
    pub fn assert_no_message_shown(&self) {
        for effect in lock(&self.screens.effects).iter() {
            assert!(
                !matches!(effect, Effect::ShowMessage { .. }),
                "unexpected {effect:?}"
            );
        }
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

#[track_caller]
pub fn assert_within_budget(took: Duration, budget: Duration) {
    assert!(took < budget, "took {took:?}, budget {budget:?}");
}

pub fn no_folder_chosen() -> LibraryViewModel {
    without_library(listing("No folder chosen", &[]))
}

pub fn no_folder_chosen_because(problem: &str) -> LibraryViewModel {
    because(no_folder_chosen(), problem)
}

pub fn because(mut screen: LibraryViewModel, problem: &str) -> LibraryViewModel {
    screen.problem = Some(problem.to_owned());

    screen
}

pub fn scanning() -> LibraryViewModel {
    let mut screen = without_library(listing("Scanning…", &[]));
    screen.choose_folder.is_enabled = false;

    screen
}

fn without_library(mut screen: LibraryViewModel) -> LibraryViewModel {
    screen.rescan.is_enabled = false;
    screen.filters = None;

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
        rescan: Action {
            label: "Rescan".to_owned(),
            is_enabled: true,
        },
        filters: Some(Filters {
            artist: field("Artist", "", "Clear Artist"),
            song: field("Song", "", "Clear Song"),
            match_count: format!("{0} / {0}", song_titles.len()),
            nothing_matches: None,
        }),
        songs: song_items(song_titles),
    }
}

#[must_use]
pub fn field(label: &str, text: &str, clear_label: &str) -> FilterField {
    FilterField {
        label: label.to_owned(),
        text: text.to_owned(),
        clear: Action {
            label: clear_label.to_owned(),
            is_enabled: false,
        },
        suggestions: vec![],
    }
}

pub trait FieldShown {
    #[must_use]
    fn clearable(self) -> Self;
    #[must_use]
    fn suggesting(self, suggestions: &[&str]) -> Self;
}

impl FieldShown for FilterField {
    fn clearable(mut self) -> Self {
        self.clear.is_enabled = true;

        self
    }

    fn suggesting(mut self, suggestions: &[&str]) -> Self {
        self.suggestions = owned_strings(suggestions);

        self
    }
}

fn owned_strings(texts: &[&str]) -> Vec<String> {
    let mut owned = Vec::new();
    for text in texts {
        owned.push((*text).to_owned());
    }

    owned
}

fn song_items(titles: &[&str]) -> Vec<SongItem> {
    let mut items = Vec::new();
    for title in titles {
        items.push(row(title));
    }

    items
}

#[must_use]
pub fn row(title: &str) -> SongItem {
    SongItem {
        title: title.to_owned(),
        artist_and_album: None,
    }
}

pub trait SongRow {
    #[must_use]
    fn above(self, artist_and_album: &str) -> Self;
}

impl SongRow for SongItem {
    fn above(mut self, artist_and_album: &str) -> Self {
        self.artist_and_album = Some(artist_and_album.to_owned());

        self
    }
}

#[must_use]
pub fn song(title: &str) -> SongInFolder {
    SongInFolder(Song::titled(title))
}

#[derive(Debug)]
pub struct SongInFolder(Song);

impl SongInFolder {
    #[must_use]
    pub fn by(mut self, artist: &str) -> Self {
        self.0.artist = Some(artist.to_owned());

        self
    }

    #[must_use]
    pub fn on(mut self, album: &str) -> Self {
        self.0.album = Some(album.to_owned());

        self
    }
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
        self.hold(location, songs);
    }

    fn hold(&self, location: &str, songs: Vec<Song>) {
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

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fluent_bundle as _;
use mt_tab_library as _;
use unic_langid as _;

use mt_tabfinder_application::{
    FolderFile, LibraryFolder, LibraryFolderGone, RememberedLibraryLocation,
};
use mt_tabfinder_presentation::{
    Action, App, AppViewModel, Effect, LibraryViewModel, SongRow, ViewModelListener,
};

type Folders = Vec<(&'static str, Vec<&'static str>)>;

#[derive(Clone, Default)]
struct FakeLibraryFolder(Arc<Mutex<Folders>>);

impl FakeLibraryFolder {
    fn with(self, location: &'static str, files: Vec<&'static str>) -> Self {
        self.folders().push((location, files));
        self
    }

    fn remove(&self, location: &str) {
        self.folders().retain(|(known, _)| *known != location);
    }

    fn folders(&self) -> MutexGuard<'_, Folders> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl LibraryFolder for FakeLibraryFolder {
    fn files(&self, location: &str) -> Result<Vec<FolderFile>, LibraryFolderGone> {
        self.folders()
            .iter()
            .find(|(known, _)| *known == location)
            .map(|(_, files)| {
                files
                    .iter()
                    .map(|path| FolderFile {
                        id: (*path).to_owned(),
                        file_name: path.rsplit('/').next().unwrap_or(path).to_owned(),
                    })
                    .collect()
            })
            .ok_or(LibraryFolderGone)
    }
}

#[derive(Clone, Default)]
struct FakeRememberedLibraryLocation(Arc<Mutex<Option<String>>>);

impl FakeRememberedLibraryLocation {
    fn of(location: &str) -> Self {
        let remembered = Self::default();
        remembered.set(Some(location.to_owned()));
        remembered
    }

    fn get(&self) -> Option<String> {
        self.slot().clone()
    }

    fn set(&self, location: Option<String>) {
        *self.slot() = location;
    }

    fn slot(&self) -> MutexGuard<'_, Option<String>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl RememberedLibraryLocation for FakeRememberedLibraryLocation {
    fn remembered(&self) -> Option<String> {
        self.get()
    }

    fn remember(&self, location: &str) {
        self.set(Some(location.to_owned()));
    }

    fn forget(&self) {
        self.set(None);
    }
}

fn app_with(folder: FakeLibraryFolder) -> (App, Recorded) {
    app_remembering(folder, FakeRememberedLibraryLocation::default())
}

fn app_remembering(
    folder: FakeLibraryFolder,
    remembered: FakeRememberedLibraryLocation,
) -> (App, Recorded) {
    let recorded = Recorded::default();
    let app = App::new(
        Box::new(recorded.clone()),
        Box::new(folder),
        Box::new(remembered),
    );
    app.start();
    (app, recorded)
}

fn choose_folder_action() -> Action {
    Action {
        label: "Choose folder".to_owned(),
        is_enabled: true,
    }
}

fn row(id: &str, title: &str) -> SongRow {
    SongRow {
        id: id.to_owned(),
        title: title.to_owned(),
    }
}

#[derive(Clone, Default)]
struct Recorded {
    view_models: Arc<Mutex<Vec<AppViewModel>>>,
    effects: Arc<Mutex<Vec<Effect>>>,
}

impl ViewModelListener for Recorded {
    fn on_view_model(&self, view_model: AppViewModel) {
        self.view_models
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(view_model);
    }

    fn on_effect(&self, effect: Effect) {
        self.effects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(effect);
    }
}

impl Recorded {
    fn last_view_model(&self) -> Option<AppViewModel> {
        self.view_models
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .last()
            .cloned()
    }

    fn view_models(&self) -> Vec<AppViewModel> {
        self.view_models
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn effects(&self) -> Vec<Effect> {
        self.effects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[test]
fn app_start_without_a_remembered_library_location_shows_no_folder_chosen() {
    let (_app, recorded) = app_with(FakeLibraryFolder::default());

    assert_eq!(
        recorded.last_view_model(),
        Some(AppViewModel {
            library: LibraryViewModel {
                status: "No folder chosen".to_owned(),
                choose_folder: choose_folder_action(),
                songs: vec![],
            },
        })
    );
}

#[test]
fn choose_folder_asks_the_platform_to_pick_a_folder() {
    let (app, recorded) = app_with(FakeLibraryFolder::default());

    app.choose_folder();

    assert_eq!(recorded.effects(), vec![Effect::PickFolder]);
}

#[test]
fn a_picked_folder_with_one_tab_file_shows_one_song_titled_by_its_file_name() {
    let (app, recorded) =
        app_with(FakeLibraryFolder::default().with("/music", vec!["/music/Wonderwall.gp5"]));

    app.folder_picked("/music");

    assert_eq!(
        recorded.last_view_model(),
        Some(AppViewModel {
            library: LibraryViewModel {
                status: "1 Song".to_owned(),
                choose_folder: choose_folder_action(),
                songs: vec![row("/music/Wonderwall.gp5", "Wonderwall")],
            },
        })
    );
}

#[test]
fn only_tab_files_become_songs_whatever_the_case_of_their_extension() {
    let (app, recorded) = app_with(FakeLibraryFolder::default().with(
        "/music",
        vec![
            "/music/a.gp3",
            "/music/b.GP4",
            "/music/c.gp5",
            "/music/d.Gpx",
            "/music/e.gp",
            "/music/f.TG",
            "/music/g.ptb",
            "/music/notes.txt",
            "/music/cover.jpg",
            "/music/gp5",
        ],
    ));

    app.folder_picked("/music");

    let library = recorded
        .last_view_model()
        .map(|view_model| view_model.library);
    assert_eq!(
        library.map(|library| (library.status, library.songs)),
        Some((
            "7 Songs".to_owned(),
            vec![
                row("/music/a.gp3", "a"),
                row("/music/b.GP4", "b"),
                row("/music/c.gp5", "c"),
                row("/music/d.Gpx", "d"),
                row("/music/e.gp", "e"),
                row("/music/f.TG", "f"),
                row("/music/g.ptb", "g"),
            ]
        ))
    );
}

#[test]
fn songs_are_sorted_by_title_ignoring_case_and_equal_titles_stay_apart() {
    let (app, recorded) = app_with(FakeLibraryFolder::default().with(
        "/music",
        vec![
            "/music/b/Wonderwall.gp5",
            "/music/angie.gp4",
            "/music/a/Wonderwall.gp5",
            "/music/Blackbird.gp5",
        ],
    ));

    app.folder_picked("/music");

    assert_eq!(
        recorded
            .last_view_model()
            .map(|view_model| view_model.library.songs),
        Some(vec![
            row("/music/angie.gp4", "angie"),
            row("/music/Blackbird.gp5", "Blackbird"),
            row("/music/a/Wonderwall.gp5", "Wonderwall"),
            row("/music/b/Wonderwall.gp5", "Wonderwall"),
        ])
    );
}

#[test]
fn a_picked_folder_without_tab_files_shows_no_tab_files_found() {
    let (app, recorded) =
        app_with(FakeLibraryFolder::default().with("/music", vec!["/music/notes.txt"]));

    app.folder_picked("/music");

    assert_eq!(
        recorded.last_view_model(),
        Some(AppViewModel {
            library: LibraryViewModel {
                status: "No Tab files found".to_owned(),
                choose_folder: choose_folder_action(),
                songs: vec![],
            },
        })
    );
}

#[test]
fn a_picked_folder_shows_scanning_until_the_scan_is_done() {
    let (app, recorded) =
        app_with(FakeLibraryFolder::default().with("/music", vec!["/music/Wonderwall.gp5"]));

    app.folder_picked("/music");

    let statuses: Vec<(String, bool)> = recorded
        .view_models()
        .into_iter()
        .map(|view_model| {
            (
                view_model.library.status,
                view_model.library.choose_folder.is_enabled,
            )
        })
        .collect();
    assert_eq!(
        statuses,
        vec![
            ("No folder chosen".to_owned(), true),
            ("Scanning…".to_owned(), false),
            ("1 Song".to_owned(), true),
        ]
    );
}

#[test]
fn cancelling_the_folder_pick_keeps_the_library() {
    let (app, recorded) =
        app_with(FakeLibraryFolder::default().with("/music", vec!["/music/Wonderwall.gp5"]));
    app.folder_picked("/music");
    let before = recorded.view_models();

    app.choose_folder();
    app.folder_pick_cancelled();

    assert_eq!(recorded.view_models(), before);
}

#[test]
fn a_picked_folder_that_is_gone_shows_no_folder_chosen() {
    let (app, recorded) = app_with(FakeLibraryFolder::default());

    app.folder_picked("/gone");

    assert_eq!(
        recorded
            .last_view_model()
            .map(|view_model| view_model.library.status),
        Some("No folder chosen".to_owned())
    );
}

#[test]
fn choosing_another_folder_replaces_the_library() {
    let (app, recorded) = app_with(
        FakeLibraryFolder::default()
            .with("/old", vec!["/old/Angie.gp4"])
            .with("/new", vec!["/new/Blackbird.gp5"]),
    );
    app.folder_picked("/old");

    let before = recorded.view_models().len();

    app.folder_picked("/new");

    let second_pick: Vec<(String, Vec<SongRow>)> = recorded.view_models()[before..]
        .iter()
        .map(|view_model| {
            (
                view_model.library.status.clone(),
                view_model.library.songs.clone(),
            )
        })
        .collect();
    assert_eq!(
        second_pick,
        vec![
            ("Scanning…".to_owned(), vec![]),
            (
                "1 Song".to_owned(),
                vec![row("/new/Blackbird.gp5", "Blackbird")]
            ),
        ]
    );
}

#[test]
fn app_start_with_a_remembered_library_location_scans_it_and_shows_its_songs() {
    let (_app, recorded) = app_remembering(
        FakeLibraryFolder::default().with("/music", vec!["/music/Wonderwall.gp5"]),
        FakeRememberedLibraryLocation::of("/music"),
    );

    assert_eq!(
        recorded.last_view_model(),
        Some(AppViewModel {
            library: LibraryViewModel {
                status: "1 Song".to_owned(),
                choose_folder: choose_folder_action(),
                songs: vec![row("/music/Wonderwall.gp5", "Wonderwall")],
            },
        })
    );
}

#[test]
fn app_start_with_a_remembered_library_location_goes_straight_to_scanning() {
    let (_app, recorded) = app_remembering(
        FakeLibraryFolder::default().with("/music", vec!["/music/Wonderwall.gp5"]),
        FakeRememberedLibraryLocation::of("/music"),
    );

    let statuses: Vec<String> = recorded
        .view_models()
        .into_iter()
        .map(|view_model| view_model.library.status)
        .collect();
    assert_eq!(statuses, vec!["Scanning…".to_owned(), "1 Song".to_owned()]);
}

#[test]
fn a_picked_folder_becomes_the_remembered_library_location() {
    let remembered = FakeRememberedLibraryLocation::default();
    let (app, _recorded) = app_remembering(
        FakeLibraryFolder::default().with("/music", vec!["/music/Wonderwall.gp5"]),
        remembered.clone(),
    );

    app.folder_picked("/music");

    assert_eq!(remembered.get(), Some("/music".to_owned()));
}

#[test]
fn app_start_with_a_remembered_library_location_that_is_gone_forgets_it_and_shows_no_folder_chosen()
{
    let remembered = FakeRememberedLibraryLocation::of("/gone");
    let (_app, recorded) = app_remembering(FakeLibraryFolder::default(), remembered.clone());

    assert_eq!(
        (
            recorded
                .last_view_model()
                .map(|view_model| view_model.library.status),
            remembered.get()
        ),
        (Some("No folder chosen".to_owned()), None)
    );
}

#[test]
fn picking_a_folder_that_is_gone_keeps_the_remembered_library_location() {
    let remembered = FakeRememberedLibraryLocation::of("/music");
    let (app, _recorded) = app_remembering(
        FakeLibraryFolder::default().with("/music", vec!["/music/Wonderwall.gp5"]),
        remembered.clone(),
    );

    app.folder_picked("/gone");

    assert_eq!(remembered.get(), Some("/music".to_owned()));
}

#[test]
fn a_scan_of_the_remembered_library_location_after_its_folder_is_gone_forgets_it() {
    let folder = FakeLibraryFolder::default().with("/music", vec!["/music/Wonderwall.gp5"]);
    let remembered = FakeRememberedLibraryLocation::of("/music");
    let (app, recorded) = app_remembering(folder.clone(), remembered.clone());
    folder.remove("/music");

    app.folder_picked("/music");

    assert_eq!(
        (
            recorded
                .last_view_model()
                .map(|view_model| view_model.library.status),
            remembered.get()
        ),
        (Some("No folder chosen".to_owned()), None)
    );
}

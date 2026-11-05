use std::fmt;

use fluent_bundle::{FluentArgs, FluentResource, concurrent::FluentBundle};
use mt_tab_library::Library;
use mt_tabfinder_application::{LibraryFolder, RememberedLibraryLocation, open_library};
use unic_langid::LanguageIdentifier;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppViewModel {
    pub library: LibraryViewModel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryViewModel {
    pub status: String,
    pub choose_folder: Action,
    pub songs: Vec<SongRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SongRow {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub label: String,
    pub is_enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    PickFolder,
}

pub trait ViewModelListener: Send + Sync {
    fn on_view_model(&self, view_model: AppViewModel);
    fn on_effect(&self, effect: Effect);
}

pub struct App {
    listener: Box<dyn ViewModelListener>,
    library_folder: Box<dyn LibraryFolder>,
    remembered_library_location: Box<dyn RememberedLibraryLocation>,
    texts: Texts,
}

impl fmt::Debug for App {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("App").finish_non_exhaustive()
    }
}

impl App {
    #[must_use]
    pub fn new(
        listener: Box<dyn ViewModelListener>,
        library_folder: Box<dyn LibraryFolder>,
        remembered_library_location: Box<dyn RememberedLibraryLocation>,
    ) -> Self {
        Self {
            listener,
            library_folder,
            remembered_library_location,
            texts: Texts::english(),
        }
    }

    pub fn start(&self) {
        match self.remembered_library_location.remembered() {
            Some(location) => self.folder_picked(&location),
            None => self.listener.on_view_model(self.no_folder_chosen()),
        }
    }

    pub fn choose_folder(&self) {
        self.listener.on_effect(Effect::PickFolder);
    }

    pub fn folder_picked(&self, location: &str) {
        self.listener.on_view_model(self.scanning());
        let view_model = open_library(
            self.library_folder.as_ref(),
            self.remembered_library_location.as_ref(),
            location,
        )
        .map_or_else(|_| self.no_folder_chosen(), |library| self.songs(&library));
        self.listener.on_view_model(view_model);
    }

    pub const fn folder_pick_cancelled(&self) {}

    fn no_folder_chosen(&self) -> AppViewModel {
        self.library_screen(self.texts.get("no-folder-chosen"), vec![])
    }

    fn scanning(&self) -> AppViewModel {
        let mut view_model = self.library_screen(self.texts.get("scanning"), vec![]);
        view_model.library.choose_folder.is_enabled = false;
        view_model
    }

    fn songs(&self, library: &Library) -> AppViewModel {
        if library.songs().is_empty() {
            return self.library_screen(self.texts.get("no-tab-files-found"), vec![]);
        }
        let mut count = FluentArgs::new();
        count.set("count", library.songs().len());
        self.library_screen(
            self.texts.get_with("songs-count", &count),
            library
                .songs()
                .iter()
                .map(|song| SongRow {
                    id: song.id.clone(),
                    title: song.title.clone(),
                })
                .collect(),
        )
    }

    fn library_screen(&self, status: String, songs: Vec<SongRow>) -> AppViewModel {
        AppViewModel {
            library: LibraryViewModel {
                status,
                choose_folder: Action {
                    label: self.texts.get("choose-folder"),
                    is_enabled: true,
                },
                songs,
            },
        }
    }
}

struct Texts(FluentBundle<FluentResource>);

impl Texts {
    fn english() -> Self {
        let english: LanguageIdentifier = "en".parse().unwrap_or_default();
        let resource = FluentResource::try_new(include_str!("../locales/en.ftl").to_owned())
            .unwrap_or_else(|(partially_parsed, _)| partially_parsed);
        let mut bundle = FluentBundle::new_concurrent(vec![english]);
        bundle.set_use_isolating(false);
        bundle.add_resource_overriding(resource);
        Self(bundle)
    }

    fn get(&self, id: &str) -> String {
        self.format(id, None)
    }

    fn get_with(&self, id: &str, args: &FluentArgs<'_>) -> String {
        self.format(id, Some(args))
    }

    fn format(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        self.0
            .get_message(id)
            .and_then(|message| message.value())
            .map_or_else(
                || id.to_owned(),
                |pattern| {
                    self.0
                        .format_pattern(pattern, args, &mut vec![])
                        .into_owned()
                },
            )
    }
}

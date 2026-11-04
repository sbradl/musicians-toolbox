use std::fmt;

use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use unic_langid::LanguageIdentifier;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppViewModel {
    pub library: LibraryViewModel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryViewModel {
    pub status: String,
    pub choose_folder: Action,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub label: String,
    pub is_enabled: bool,
}

pub trait ViewModelListener: Send + Sync {
    fn on_view_model(&self, view_model: AppViewModel);
}

pub struct App {
    listener: Box<dyn ViewModelListener>,
    texts: Texts,
}

impl fmt::Debug for App {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("App").finish_non_exhaustive()
    }
}

impl App {
    #[must_use]
    pub fn new(listener: Box<dyn ViewModelListener>) -> Self {
        Self {
            listener,
            texts: Texts::english(),
        }
    }

    pub fn start(&self) {
        self.listener.on_view_model(self.no_folder_chosen());
    }

    pub const fn choose_folder(&self) {}

    fn no_folder_chosen(&self) -> AppViewModel {
        AppViewModel {
            library: LibraryViewModel {
                status: self.texts.get("no-folder-chosen"),
                choose_folder: Action {
                    label: self.texts.get("choose-folder"),
                    is_enabled: true,
                },
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
        bundle.add_resource_overriding(resource);
        Self(bundle)
    }

    fn get(&self, id: &str) -> String {
        self.0
            .get_message(id)
            .and_then(|message| message.value())
            .map_or_else(
                || id.to_owned(),
                |pattern| {
                    self.0
                        .format_pattern(pattern, None, &mut vec![])
                        .into_owned()
                },
            )
    }
}

mod texts;

use texts::Texts;

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

    pub fn choose_folder(&self) {}

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

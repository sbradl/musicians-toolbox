use std::fmt;

use mt_tabfinder_presentation as presentation;
use presentation::{Action, AppViewModel, LibraryViewModel};

uniffi::setup_scaffolding!();

#[uniffi::remote(Record)]
pub struct AppViewModel {
    pub library: LibraryViewModel,
}

#[uniffi::remote(Record)]
pub struct LibraryViewModel {
    pub status: String,
    pub choose_folder: Action,
}

#[uniffi::remote(Record)]
pub struct Action {
    pub label: String,
    pub is_enabled: bool,
}

#[uniffi::export(callback_interface)]
pub trait ViewModelListener: Send + Sync {
    fn on_view_model(&self, view_model: AppViewModel);
}

struct ForeignViewModelListener(Box<dyn ViewModelListener>);

impl presentation::ViewModelListener for ForeignViewModelListener {
    fn on_view_model(&self, view_model: AppViewModel) {
        self.0.on_view_model(view_model);
    }
}

#[derive(uniffi::Object)]
pub struct App(presentation::App);

impl fmt::Debug for App {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("App").field(&self.0).finish()
    }
}

#[uniffi::export]
impl App {
    #[uniffi::constructor]
    #[must_use]
    pub fn new(listener: Box<dyn ViewModelListener>) -> Self {
        Self(presentation::App::new(Box::new(ForeignViewModelListener(
            listener,
        ))))
    }

    pub fn start(&self) {
        self.0.start();
    }

    pub const fn choose_folder(&self) {
        self.0.choose_folder();
    }
}

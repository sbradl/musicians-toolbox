use std::sync::{Arc, Mutex};

use mt_tabfinder_presentation::{Action, App, AppViewModel, LibraryViewModel, ViewModelListener};

#[derive(Clone, Default)]
struct RecordedViewModels(Arc<Mutex<Vec<AppViewModel>>>);

impl ViewModelListener for RecordedViewModels {
    fn on_view_model(&self, view_model: AppViewModel) {
        self.0.lock().unwrap().push(view_model);
    }
}

impl RecordedViewModels {
    fn last(&self) -> Option<AppViewModel> {
        self.0.lock().unwrap().last().cloned()
    }
}

#[test]
fn app_start_without_a_remembered_library_location_shows_no_folder_chosen() {
    let view_models = RecordedViewModels::default();
    let app = App::new(Box::new(view_models.clone()));

    app.start();

    assert_eq!(
        view_models.last(),
        Some(AppViewModel {
            library: LibraryViewModel {
                status: "No folder chosen".to_owned(),
                choose_folder: Action {
                    label: "Choose folder".to_owned(),
                    is_enabled: true,
                },
            },
        })
    );
}

use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Library {
    songs: Vec<Song>,
}

impl Library {
    pub fn scanned(files: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut songs: Vec<Song> = files
            .into_iter()
            .filter(|(_, file_name)| is_tab_file(file_name))
            .map(|(id, file_name)| Song {
                id,
                title: title_of(&file_name),
            })
            .collect();
        songs.sort_by_cached_key(|song| (song.title.to_lowercase(), song.id.clone()));
        Self { songs }
    }

    #[must_use]
    pub fn songs(&self) -> &[Song] {
        &self.songs
    }
}

const TAB_FILE_EXTENSIONS: [&str; 7] = ["gp3", "gp4", "gp5", "gpx", "gp", "tg", "ptb"];

fn is_tab_file(file_name: &str) -> bool {
    Path::new(file_name).extension().is_some_and(|extension| {
        TAB_FILE_EXTENSIONS
            .iter()
            .any(|tab| extension.eq_ignore_ascii_case(tab))
    })
}

fn title_of(file_name: &str) -> String {
    Path::new(file_name).file_stem().map_or_else(
        || file_name.to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

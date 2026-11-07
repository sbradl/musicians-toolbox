use mt_library::{Library, Song};

use crate::{Action, LibraryViewModel, SongItem, Texts};

impl LibraryViewModel {
    pub(super) fn no_folder_chosen(texts: &Texts) -> Self {
        Self::with_status(texts.no_folder_chosen(), texts)
    }

    pub(super) fn scanning(texts: &Texts) -> Self {
        Self::with_status(texts.scanning(), texts).with_choose_folder_disabled()
    }

    pub(super) fn listing(library: &Library, texts: &Texts) -> Self {
        let songs = library.songs();

        if songs.is_empty() {
            return Self::with_status(texts.no_songs_found(), texts);
        }

        Self::with_status(texts.songs_count(songs.len()), texts).with_songs(songs)
    }

    fn with_status(status: String, texts: &Texts) -> Self {
        Self {
            status,
            choose_folder: Action {
                label: texts.choose_folder(),
                is_enabled: true,
            },
            songs: vec![],
        }
    }

    const fn with_choose_folder_disabled(mut self) -> Self {
        self.choose_folder.is_enabled = false;

        self
    }

    fn with_songs(mut self, songs: &[Song]) -> Self {
        self.songs = songs.iter().map(SongItem::from).collect();

        self
    }
}

impl From<&Song> for SongItem {
    fn from(song: &Song) -> Self {
        Self {
            title: song.title.clone(),
        }
    }
}

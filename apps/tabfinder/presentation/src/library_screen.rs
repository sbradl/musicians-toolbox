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

        Self::with_status(Self::songs_found(songs, texts), texts)
            .with_rescan_enabled()
            .with_songs(songs, texts)
    }

    fn songs_found(songs: &[Song], texts: &Texts) -> String {
        if songs.is_empty() {
            return texts.no_songs_found();
        }

        texts.songs_count(songs.len())
    }

    pub(super) fn because(mut self, problem: String) -> Self {
        self.problem = Some(problem);

        self
    }

    fn with_status(status: String, texts: &Texts) -> Self {
        Self {
            status,
            problem: None,
            choose_folder: Action {
                label: texts.choose_folder(),
                is_enabled: true,
            },
            rescan: Action {
                label: texts.rescan(),
                is_enabled: false,
            },
            songs: vec![],
        }
    }

    const fn with_rescan_enabled(mut self) -> Self {
        self.rescan.is_enabled = true;

        self
    }

    const fn with_choose_folder_disabled(mut self) -> Self {
        self.choose_folder.is_enabled = false;

        self
    }

    fn with_songs(mut self, songs: &[Song], texts: &Texts) -> Self {
        let mut items = Vec::new();
        for song in songs {
            items.push(SongItem::of(song, texts));
        }
        self.songs = items;

        self
    }
}

impl SongItem {
    fn of(song: &Song, texts: &Texts) -> Self {
        Self {
            title: song.title.clone(),
            artist_and_album: Self::artist_and_album(song, texts),
        }
    }

    fn artist_and_album(song: &Song, texts: &Texts) -> Option<String> {
        let artist = song.artist.as_deref()?;
        let Some(album) = song.album.as_deref() else {
            return Some(artist.to_owned());
        };

        Some(texts.artist_and_album(artist, album))
    }
}

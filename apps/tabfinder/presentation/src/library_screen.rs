use mt_library::{Filter, Library, Song};

use crate::{
    Action, FilterField, Filters, LibraryViewModel, ListedLibrary, NothingMatches, SongItem, Texts,
};

impl LibraryViewModel {
    pub(super) fn no_folder_chosen(texts: &Texts) -> Self {
        Self::with_status(texts.no_folder_chosen(), texts)
    }

    pub(super) fn scanning(texts: &Texts) -> Self {
        Self::with_status(texts.scanning(), texts).with_choose_folder_disabled()
    }

    pub(super) fn listing(listed: &ListedLibrary, filter: &Filter, texts: &Texts) -> Self {
        let all_songs = listed.library.songs();
        let matching = listed.items_matching(filter);
        let filters = Filters::of(filter, matching.len(), all_songs.len(), texts);

        Self::with_status(Self::songs_found(all_songs, texts), texts)
            .with_rescan_enabled()
            .with_filters(filters)
            .with_songs(matching)
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
            filters: None,
            songs: vec![],
        }
    }

    fn with_filters(mut self, filters: Filters) -> Self {
        self.filters = Some(filters);

        self
    }

    const fn with_rescan_enabled(mut self) -> Self {
        self.rescan.is_enabled = true;

        self
    }

    const fn with_choose_folder_disabled(mut self) -> Self {
        self.choose_folder.is_enabled = false;

        self
    }

    fn with_songs(mut self, songs: Vec<SongItem>) -> Self {
        self.songs = songs;

        self
    }
}

impl ListedLibrary {
    pub(super) fn of(library: Library, texts: &Texts) -> Self {
        let mut items = Vec::new();
        for song in library.songs() {
            items.push(SongItem::of(song, texts));
        }

        Self { library, items }
    }

    fn items_matching(&self, filter: &Filter) -> Vec<SongItem> {
        let mut matching = Vec::new();
        for (song, item) in self.library.songs().iter().zip(&self.items) {
            if filter.matches(song) {
                matching.push(item.clone());
            }
        }

        matching
    }
}

impl Filters {
    fn of(filter: &Filter, matching: usize, all: usize, texts: &Texts) -> Self {
        Self {
            song: FilterField {
                label: texts.song_filter(),
                text: filter.song_text().to_owned(),
                clear: Action {
                    label: texts.clear_song_filter(),
                    is_enabled: !filter.song_text().is_empty(),
                },
            },
            match_count: texts.songs_matching(matching, all),
            nothing_matches: Self::nothing_matches(matching, all, texts),
        }
    }

    fn nothing_matches(matching: usize, all: usize, texts: &Texts) -> Option<NothingMatches> {
        if matching > 0 || all == 0 {
            return None;
        }

        Some(NothingMatches {
            text: texts.no_songs_match(),
            clear_filters: texts.clear_filters(),
        })
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

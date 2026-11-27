use mt_library::{Filter, FilterBy, Library, Song};

use crate::{
    Action, FilterField, Filters, LibraryViewModel, ListedLibrary, NothingMatches, SongItem,
    SuggestionGroup, Texts,
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
        let filters = Filters::of(listed, filter, matching.len(), all_songs.len(), texts);

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

    fn suggestions_for(
        &self,
        by: FilterBy,
        filter: &Filter,
        texts: &Texts,
    ) -> Option<Vec<SuggestionGroup>> {
        match by {
            FilterBy::Artist => Some(self.artists_suggested_by(filter)),
            FilterBy::Song => None,
            FilterBy::Tuning => Some(self.tunings_suggested_by(filter, texts)),
        }
    }

    fn artists_suggested_by(&self, filter: &Filter) -> Vec<SuggestionGroup> {
        let mut suggested = Vec::new();
        for artist in self.library.artists_suggested_by(filter) {
            suggested.push(artist.to_owned());
        }

        if suggested.is_empty() {
            return Vec::new();
        }

        vec![SuggestionGroup {
            heading: None,
            suggestions: suggested,
        }]
    }

    fn tunings_suggested_by(&self, filter: &Filter, texts: &Texts) -> Vec<SuggestionGroup> {
        let mut groups: Vec<SuggestionGroup> = Vec::new();
        let mut strings_of_last_group = None;
        for tuning in self.library.tunings_suggested_by(filter) {
            if strings_of_last_group != Some(tuning.strings()) {
                strings_of_last_group = Some(tuning.strings());
                groups.push(SuggestionGroup::headed(
                    texts.tuning_group(tuning.strings()),
                ));
            }

            if let Some(group) = groups.last_mut() {
                group.suggestions.push(texts.tuning_badge(tuning));
            }
        }

        groups
    }

    fn items_matching(&self, filter: &Filter) -> Vec<SongItem> {
        let mut matching = Vec::new();
        for (song, item) in self.library.songs().iter().zip(&self.items) {
            if self.library.matches(filter, song) {
                matching.push(item.clone());
            }
        }

        matching
    }
}

impl Filters {
    fn of(
        listed: &ListedLibrary,
        filter: &Filter,
        matching: usize,
        all: usize,
        texts: &Texts,
    ) -> Self {
        let mut fields = Vec::new();
        for by in FilterBy::ALL {
            let suggestions = listed.suggestions_for(by, filter, texts);
            fields.push(FilterField::of(by, filter.text(by), suggestions, texts));
        }

        Self {
            fields,
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

impl FilterField {
    fn of(
        by: FilterBy,
        text: &str,
        suggestions: Option<Vec<SuggestionGroup>>,
        texts: &Texts,
    ) -> Self {
        Self {
            by,
            label: texts.filter_label(by),
            text: text.to_owned(),
            clear: Action {
                label: texts.clear_filter(by),
                is_enabled: !text.is_empty(),
            },
            suggests: suggestions.is_some(),
            suggestions: suggestions.unwrap_or_default(),
        }
    }
}

impl SuggestionGroup {
    const fn headed(heading: String) -> Self {
        Self {
            heading: Some(heading),
            suggestions: Vec::new(),
        }
    }
}

impl SongItem {
    fn of(song: &Song, texts: &Texts) -> Self {
        Self {
            title: song.title.clone(),
            artist_and_album: Self::artist_and_album(song, texts),
            tunings: Self::badges(song, texts),
            unreadable: Self::unreadable(song, texts),
        }
    }

    fn badges(song: &Song, texts: &Texts) -> Vec<String> {
        let mut badges = Vec::new();
        for tuning in &song.tunings {
            badges.push(texts.tuning_badge(tuning));
        }

        badges
    }

    fn unreadable(song: &Song, texts: &Texts) -> Option<String> {
        if song.is_unreadable {
            return Some(texts.unreadable_file());
        }

        None
    }

    fn artist_and_album(song: &Song, texts: &Texts) -> Option<String> {
        let artist = song.artist.as_deref()?;
        let Some(album) = song.album.as_deref() else {
            return Some(artist.to_owned());
        };

        Some(texts.artist_and_album(artist, album))
    }
}

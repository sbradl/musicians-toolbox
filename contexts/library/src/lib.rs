use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use mt_music_kernel::{Tuning, WrittenTempo};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub tunings: Vec<Tuning>,
    pub written_tempos: Vec<WrittenTempo>,
    pub is_unreadable: bool,
}

impl Song {
    pub fn titled(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            artist: None,
            album: None,
            tunings: Vec::new(),
            written_tempos: Vec::new(),
            is_unreadable: false,
        }
    }
}

#[derive(Debug)]
pub struct Library {
    songs: Vec<Song>,
    tunings: TuningTexts,
}

impl Library {
    pub fn of(mut songs: Vec<Song>) -> Self {
        songs.sort_by_cached_key(|song| song.title.to_lowercase());
        spell_names_as_most_songs_do(&mut songs);
        let tunings = TuningTexts::of(&songs);

        Self { songs, tunings }
    }

    pub fn matches(&self, filter: &Filter, song: &Song) -> bool {
        filter.matches(song, &self.tunings)
    }

    pub fn songs(&self) -> &[Song] {
        &self.songs
    }

    pub fn unreadable_songs(&self) -> usize {
        self.songs.iter().filter(|song| song.is_unreadable).count()
    }

    pub fn artists_suggested_by(&self, filter: &Filter) -> Vec<&str> {
        let mut artists = Vec::new();
        for song in &self.songs {
            if let Some(artist) = filter.suggested_artist_of(song, &self.tunings) {
                artists.push(artist);
            }
        }
        artists.sort_by_cached_key(|artist| (artist.to_lowercase(), *artist));
        artists.dedup();

        artists
    }

    pub fn tunings_suggested_by(&self, filter: &Filter) -> Vec<&Tuning> {
        let mut distinct = HashSet::new();
        for song in &self.songs {
            distinct.extend(filter.suggested_tunings_of(song, &self.tunings));
        }
        let mut tunings: Vec<&Tuning> = distinct.into_iter().collect();
        tunings.sort_by_key(|tuning| (tuning.strings(), self.tunings.ignoring_case(tuning)));
        tunings.dedup_by_key(|tuning| self.tunings.ignoring_case(tuning));

        tunings
    }
}

fn spell_names_as_most_songs_do(songs: &mut [Song]) {
    spell_artists_as_most_songs_do(songs);
    spell_albums_as_most_songs_do(songs);
}

fn spell_artists_as_most_songs_do(songs: &mut [Song]) {
    let no_owner = "";
    let mut spellings = Spellings::default();
    for song in songs.iter() {
        spellings.count(no_owner, song.artist.as_deref());
    }
    let most_used = spellings.most_used();
    for song in songs {
        most_used.respell(no_owner, &mut song.artist);
    }
}

fn spell_albums_as_most_songs_do(songs: &mut [Song]) {
    let mut spellings = Spellings::default();
    for song in songs.iter() {
        spellings.count(artist_of(song), song.album.as_deref());
    }
    let most_used = spellings.most_used();
    for song in songs {
        most_used.respell(song.artist.as_deref().unwrap_or_default(), &mut song.album);
    }
}

fn artist_of(song: &Song) -> &str {
    song.artist.as_deref().unwrap_or_default()
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct SameName {
    owner: String,
    ignoring_case: String,
}

impl SameName {
    fn of(owner: &str, name: &str) -> Self {
        Self {
            owner: owner.to_lowercase(),
            ignoring_case: name.to_lowercase(),
        }
    }
}

#[derive(Debug, Default)]
struct Spellings {
    uses: HashMap<SameName, HashMap<String, usize>>,
}

impl Spellings {
    fn count(&mut self, owner: &str, name: Option<&str>) {
        let Some(name) = name else {
            return;
        };
        let uses = self.uses.entry(SameName::of(owner, name)).or_default();
        *uses.entry(name.to_owned()).or_default() += 1;
    }

    fn most_used(self) -> MostUsedSpellings {
        let mut most_used = HashMap::new();
        for (same_name, uses) in self.uses {
            drop(most_used.insert(same_name, most_used_of(uses)));
        }

        MostUsedSpellings(most_used)
    }
}

fn most_used_of(uses: HashMap<String, usize>) -> String {
    let mut spellings: Vec<(String, usize)> = uses.into_iter().collect();
    spellings.sort_by_cached_key(|(spelling, used)| {
        (Reverse(*used), capitals_in(spelling), spelling.clone())
    });
    let mut ranked = spellings.into_iter();

    ranked
        .next()
        .map(|(spelling, _)| spelling)
        .unwrap_or_default()
}

fn capitals_in(spelling: &str) -> usize {
    spelling
        .chars()
        .filter(|letter| letter.is_uppercase())
        .count()
}

#[derive(Debug)]
struct MostUsedSpellings(HashMap<SameName, String>);

impl MostUsedSpellings {
    fn respell(&self, owner: &str, name: &mut Option<String>) {
        let Some(spelled) = name.as_deref() else {
            return;
        };

        if let Some(most_used) = self.0.get(&SameName::of(owner, spelled)) {
            most_used.clone_into(name.get_or_insert_default());
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterBy {
    Artist,
    Song,
    Tuning,
}

impl FilterBy {
    pub const ALL: [Self; 3] = [Self::Artist, Self::Song, Self::Tuning];
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    texts: HashMap<FilterBy, FilterText>,
}

impl Filter {
    pub fn set(&mut self, by: FilterBy, text: &str) {
        if text.is_empty() {
            drop(self.texts.remove(&by));

            return;
        }

        drop(self.texts.insert(by, FilterText::of(text)));
    }

    pub fn text(&self, by: FilterBy) -> &str {
        let Some(text) = self.texts.get(&by) else {
            return "";
        };

        &text.typed
    }

    fn matches(&self, song: &Song, tunings: &TuningTexts) -> bool {
        let mut every_field = FilterBy::ALL.iter();

        every_field.all(|by| self.finds(*by, song, tunings))
    }

    fn may_suggest_from(&self, by: FilterBy, song: &Song, tunings: &TuningTexts) -> bool {
        let mut every_other_field = FilterBy::ALL.iter().filter(|other| **other != by);

        self.typed(by).is_some() && every_other_field.all(|other| self.finds(*other, song, tunings))
    }

    fn suggested_artist_of<'song>(
        &self,
        song: &'song Song,
        tunings: &TuningTexts,
    ) -> Option<&'song str> {
        let is_suggested = self.may_suggest_from(FilterBy::Artist, song, tunings)
            && self.finds(FilterBy::Artist, song, tunings);

        if is_suggested {
            return song.artist.as_deref();
        }

        None
    }

    fn suggested_tunings_of<'song>(
        &self,
        song: &'song Song,
        tunings: &TuningTexts,
    ) -> Vec<&'song Tuning> {
        let Some(text) = self.typed(FilterBy::Tuning) else {
            return Vec::new();
        };

        if !self.may_suggest_from(FilterBy::Tuning, song, tunings) {
            return Vec::new();
        }

        let mut suggested = Vec::new();
        for tuning in &song.tunings {
            if text.appears_in_tuning(tuning, tunings) {
                suggested.push(tuning);
            }
        }

        suggested
    }

    fn finds(&self, by: FilterBy, song: &Song, tunings: &TuningTexts) -> bool {
        let Some(text) = self.typed(by) else {
            return true;
        };

        match by {
            FilterBy::Artist => text.appears_in_artist_of(song),
            FilterBy::Song => text.appears_in(&song.title),
            FilterBy::Tuning => text.appears_in_a_tuning_of(song, tunings),
        }
    }

    fn typed(&self, by: FilterBy) -> Option<&FilterText> {
        self.texts.get(&by)
    }
}

#[derive(Debug, Default)]
struct TuningTexts(HashMap<Tuning, String>);

impl TuningTexts {
    fn of(songs: &[Song]) -> Self {
        let mut texts = HashMap::new();
        for tuning in songs.iter().flat_map(|song| &song.tunings) {
            if !texts.contains_key(tuning) {
                drop(texts.insert(tuning.clone(), text_ignoring_case(tuning)));
            }
        }

        Self(texts)
    }

    fn ignoring_case(&self, tuning: &Tuning) -> Cow<'_, str> {
        let Some(text) = self.0.get(tuning) else {
            return Cow::Owned(text_ignoring_case(tuning));
        };

        Cow::Borrowed(text)
    }
}

fn text_ignoring_case(tuning: &Tuning) -> String {
    format!("{} {}", tuning.strings(), tuning.name()).to_lowercase()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FilterText {
    typed: String,
    ignoring_case: String,
}

impl FilterText {
    fn of(text: &str) -> Self {
        Self {
            typed: text.to_owned(),
            ignoring_case: text.to_lowercase(),
        }
    }

    fn appears_in(&self, text: &str) -> bool {
        self.appears_in_lowercased(&text.to_lowercase())
    }

    fn appears_in_lowercased(&self, lowercased: &str) -> bool {
        lowercased.contains(&self.ignoring_case)
    }

    fn appears_in_artist_of(&self, song: &Song) -> bool {
        let Some(artist) = song.artist.as_deref() else {
            return false;
        };

        self.appears_in(artist)
    }

    fn appears_in_a_tuning_of(&self, song: &Song, tunings: &TuningTexts) -> bool {
        for tuning in &song.tunings {
            if self.appears_in_tuning(tuning, tunings) {
                return true;
            }
        }

        false
    }

    fn appears_in_tuning(&self, tuning: &Tuning, tunings: &TuningTexts) -> bool {
        self.appears_in_lowercased(&tunings.ignoring_case(tuning))
    }
}

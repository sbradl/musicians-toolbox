use std::cmp::Reverse;
use std::collections::HashMap;

use mt_music_kernel::Tuning;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub tunings: Vec<Tuning>,
    pub is_unreadable: bool,
}

impl Song {
    pub fn titled(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            artist: None,
            album: None,
            tunings: Vec::new(),
            is_unreadable: false,
        }
    }
}

#[derive(Debug)]
pub struct Library {
    songs: Vec<Song>,
}

impl Library {
    pub fn of(mut songs: Vec<Song>) -> Self {
        songs.sort_by_cached_key(|song| song.title.to_lowercase());
        spell_names_as_most_songs_do(&mut songs);

        Self { songs }
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
            if let Some(artist) = filter.suggested_artist_of(song) {
                artists.push(artist);
            }
        }
        artists.sort_by_cached_key(|artist| (artist.to_lowercase(), *artist));
        artists.dedup();

        artists
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    song: FilterText,
    artist: FilterText,
}

impl Filter {
    pub fn set_song(&mut self, text: &str) {
        self.song = FilterText::of(text);
    }

    pub fn set_artist(&mut self, text: &str) {
        self.artist = FilterText::of(text);
    }

    pub fn song_text(&self) -> &str {
        &self.song.typed
    }

    pub fn artist_text(&self) -> &str {
        &self.artist.typed
    }

    pub fn matches(&self, song: &Song) -> bool {
        self.song.appears_in(&song.title) && self.matches_artist_of(song)
    }

    fn suggested_artist_of<'song>(&self, song: &'song Song) -> Option<&'song str> {
        let artist = song.artist.as_deref()?;
        let is_suggested = !self.artist.typed.is_empty()
            && self.artist.appears_in(artist)
            && self.song.appears_in(&song.title);

        if is_suggested {
            return Some(artist);
        }

        None
    }

    fn matches_artist_of(&self, song: &Song) -> bool {
        let Some(artist) = song.artist.as_deref() else {
            return self.artist.typed.is_empty();
        };

        self.artist.appears_in(artist)
    }
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
        text.to_lowercase().contains(&self.ignoring_case)
    }
}

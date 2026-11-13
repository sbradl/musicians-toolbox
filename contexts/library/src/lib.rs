#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
}

impl Song {
    pub fn titled(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            artist: None,
            album: None,
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

        Self { songs }
    }

    pub fn songs(&self) -> &[Song] {
        &self.songs
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filter {
    song: String,
    song_ignoring_case: String,
}

impl Filter {
    pub fn song(text: &str) -> Self {
        Self {
            song: text.to_owned(),
            song_ignoring_case: text.to_lowercase(),
        }
    }

    pub fn song_text(&self) -> &str {
        &self.song
    }

    pub fn matches(&self, song: &Song) -> bool {
        song.title.to_lowercase().contains(&self.song_ignoring_case)
    }
}

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

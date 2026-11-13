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

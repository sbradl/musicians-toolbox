use std::fs::{self, File};
use std::io::{self, Write as _};
use std::path::PathBuf;

use mt_tabfinder_application::{LastScan, LibraryLocation, Song};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};

use crate::ignore_failure;

const ESCAPED: &AsciiSet = &CONTROLS.add(b'%');
const FIELD_SEPARATOR: &str = "\t";

#[derive(Debug)]
pub struct FsLastScan {
    file: PathBuf,
}

impl FsLastScan {
    pub fn in_config_dir(config_dir: impl Into<PathBuf>) -> Self {
        Self {
            file: config_dir.into().join("last-scan"),
        }
    }

    fn write(&self, contents: &str) -> io::Result<()> {
        let unfinished = self.file.with_extension("unfinished");

        if let Some(config_dir) = self.file.parent() {
            fs::create_dir_all(config_dir)?;
        }
        let mut file = File::create(&unfinished)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;

        fs::rename(unfinished, &self.file)
    }
}

impl LastScan for FsLastScan {
    fn songs_of(&self, location: &LibraryLocation) -> Option<Vec<Song>> {
        let contents = fs::read_to_string(&self.file).ok()?;
        let mut lines = contents.lines();
        let scanned_location = decoded(lines.next()?)?;

        if scanned_location != location.as_str() {
            return None;
        }

        songs_from(lines)
    }

    fn keep(&self, location: &LibraryLocation, songs: &[Song]) {
        if self.write(&contents_of(location, songs)).is_err() {
            ignore_failure(fs::remove_file(&self.file));
        }
    }
}

fn contents_of(location: &LibraryLocation, songs: &[Song]) -> String {
    let mut contents = encoded(location.as_str());
    for song in songs {
        contents.push('\n');
        contents.push_str(&line_of(song));
    }
    contents.push('\n');

    contents
}

fn line_of(song: &Song) -> String {
    let fields = [
        encoded(&song.title),
        encoded(song.artist.as_deref().unwrap_or_default()),
        encoded(song.album.as_deref().unwrap_or_default()),
    ];

    fields.join(FIELD_SEPARATOR)
}

fn songs_from<'contents>(lines: impl Iterator<Item = &'contents str>) -> Option<Vec<Song>> {
    let mut songs = Vec::new();
    for line in lines {
        songs.push(song_from_line(line)?);
    }

    Some(songs)
}

fn song_from_line(line: &str) -> Option<Song> {
    let fields: Vec<&str> = line.split(FIELD_SEPARATOR).collect();
    let [title, artist, album] = fields.as_slice() else {
        return None;
    };

    Some(Song {
        title: decoded(title)?,
        artist: non_empty(decoded(artist)?),
        album: non_empty(decoded(album)?),
    })
}

fn encoded(text: &str) -> String {
    utf8_percent_encode(text, ESCAPED).to_string()
}

fn decoded(field: &str) -> Option<String> {
    let text = percent_decode_str(field).decode_utf8().ok()?;

    Some(text.into_owned())
}

fn non_empty(text: String) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    Some(text)
}

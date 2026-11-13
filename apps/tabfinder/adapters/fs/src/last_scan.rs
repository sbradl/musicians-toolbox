use std::fs;
use std::path::PathBuf;

use mt_tabfinder_application::{LastScan, LibraryLocation, Pitch, Song, Tuning};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};

use crate::save_whole_or_nothing;

const ESCAPED: &AsciiSet = &CONTROLS.add(b'%');
const FIELD_SEPARATOR: &str = "\t";
const TUNING_SEPARATOR: &str = ",";
const PITCH_SEPARATOR: &str = " ";
const UNREADABLE: &str = "unreadable";

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
        save_whole_or_nothing(&self.file, &contents_of(location, songs));
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
        tunings_field(&song.tunings),
        unreadable_field(song.is_unreadable),
    ];

    fields.join(FIELD_SEPARATOR)
}

fn tunings_field(tunings: &[Tuning]) -> String {
    let mut fields = Vec::new();
    for tuning in tunings {
        let mut pitches = Vec::new();
        for pitch in &tuning.lowest_string_first {
            pitches.push(pitch.midi_number().to_string());
        }
        fields.push(pitches.join(PITCH_SEPARATOR));
    }

    fields.join(TUNING_SEPARATOR)
}

fn unreadable_field(is_unreadable: bool) -> String {
    if is_unreadable {
        return UNREADABLE.to_owned();
    }

    String::new()
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
    let [title, artist, album, tunings, unreadable] = fields.as_slice() else {
        return None;
    };

    Some(Song {
        title: decoded(title)?,
        artist: non_empty(decoded(artist)?),
        album: non_empty(decoded(album)?),
        tunings: tunings_from(tunings)?,
        is_unreadable: is_unreadable_from(unreadable)?,
    })
}

fn tunings_from(field: &str) -> Option<Vec<Tuning>> {
    let mut tunings = Vec::new();

    if field.is_empty() {
        return Some(tunings);
    }
    for tuning in field.split(TUNING_SEPARATOR) {
        let mut pitches = Vec::new();
        for midi_number in tuning.split(PITCH_SEPARATOR) {
            pitches.push(Pitch::midi(midi_number.parse().ok()?));
        }
        tunings.push(Tuning::of(pitches));
    }

    Some(tunings)
}

fn is_unreadable_from(field: &str) -> Option<bool> {
    match field {
        UNREADABLE => Some(true),
        "" => Some(false),
        _ => None,
    }
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

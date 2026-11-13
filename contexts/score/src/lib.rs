mod cursor;
mod gpif;
mod gpx;
mod guitar_pro;
mod zip;

use mt_music_kernel::{Pitch, Tuning};

pub const LARGEST_TAB_FILE: usize = 64 * 1024 * 1024;
pub const LARGEST_SCORE_XML: usize = 32 * 1024 * 1024;
const MOST_TRACKS: usize = 128;
const HIGHEST_MIDI_NUMBER: u8 = 127;

#[derive(Debug, PartialEq, Eq)]
pub struct ScoreHeader {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Track {
    pub tuning: Option<Tuning>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Unreadable;

pub fn read_header(tab_file: &[u8]) -> Result<ScoreHeader, Unreadable> {
    if tab_file.len() > LARGEST_TAB_FILE {
        return Err(Unreadable);
    }

    if let Some(zip) = Zip::recognized(tab_file) {
        return zip.read_header();
    }

    read_unzipped_header(tab_file)
}

fn read_unzipped_header(tab_file: &[u8]) -> Result<ScoreHeader, Unreadable> {
    if let Some(guitar_pro_6) = GuitarPro6File::recognized(tab_file) {
        return guitar_pro_6.read_header();
    }

    let Some(guitar_pro) = GuitarProFile::recognized(tab_file) else {
        return Err(Unreadable);
    };

    guitar_pro.read_header()
}

fn pitch_of(midi_number: i32) -> Result<Pitch, Unreadable> {
    let Ok(midi_number) = u8::try_from(midi_number) else {
        return Err(Unreadable);
    };

    if midi_number > HIGHEST_MIDI_NUMBER {
        return Err(Unreadable);
    }

    Ok(Pitch::midi(midi_number))
}

#[derive(Debug)]
struct Zip<'file>(&'file [u8]);

#[derive(Debug)]
struct ScoreXml<Xml>(Xml);

#[derive(Debug)]
struct GuitarPro6File<'file>(&'file [u8]);

#[derive(Debug)]
struct GuitarProFile<'file>(&'file [u8]);

#[derive(Debug)]
struct Cursor<'file> {
    tab_file: &'file [u8],
    position: usize,
}

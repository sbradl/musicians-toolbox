mod cursor;
mod gpif;
mod gpx;
mod guitar_pro;
mod open_elements;
mod power_tab;
mod tux_guitar;
mod tux_guitar_xml;
mod zip;

use mt_music_kernel::{Pitch, Tuning, WrittenTempo};

pub const LARGEST_TAB_FILE: usize = 64 * 1024 * 1024;
pub const LARGEST_SCORE_XML: usize = 32 * 1024 * 1024;
const MOST_TRACKS: usize = 128;
const MOST_STRINGS: usize = 16;
const MOST_XML_DEPTH: usize = 64;
const MOST_XML_ELEMENTS: usize = 1_000_000;
const HIGHEST_MIDI_NUMBER: u8 = 127;
const MOST_BEATS_PER_MINUTE: u16 = 1000;

#[derive(Debug, PartialEq, Eq)]
pub struct ScoreHeader {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub tracks: Vec<Track>,
    pub written_tempos: Vec<WrittenTempo>,
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

    if let Some(guitar_pro) = GuitarProFile::recognized(tab_file) {
        return guitar_pro.read_header();
    }

    if let Some(tux_guitar_1) = TuxGuitar1File::recognized(tab_file) {
        return tux_guitar_1.read_header();
    }

    let Some(power_tab) = PowerTabFile::recognized(tab_file) else {
        return Err(Unreadable);
    };

    power_tab.read_header()
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

// shortcut: Latin-1 stands in for Windows-1252 (differs in 0x80-0x9F: curly quotes, €), upgrade if such names show wrong signs.
fn decoded(text: &[u8]) -> String {
    if let Ok(utf8) = str::from_utf8(text) {
        return utf8.to_owned();
    }

    text.iter().map(|&byte| char::from(byte)).collect()
}

fn stated(name: &str) -> Option<String> {
    let name = name.trim();

    if name.is_empty() {
        return None;
    }

    Some(name.to_owned())
}

fn stated_tempo(beats_per_minute: i64) -> Option<WrittenTempo> {
    let Ok(beats_per_minute) = u16::try_from(beats_per_minute) else {
        return None;
    };

    if beats_per_minute == 0 || beats_per_minute > MOST_BEATS_PER_MINUTE {
        return None;
    }

    Some(WrittenTempo::beats_per_minute(beats_per_minute))
}

#[derive(Debug)]
struct Zip<'file>(&'file [u8]);

#[derive(Debug)]
struct ScoreXml<Xml>(Xml);

#[derive(Debug)]
struct TuxGuitarXml<Xml>(Xml);

#[derive(Debug, Default)]
struct OpenElements {
    names: Vec<String>,
    seen: usize,
}

#[derive(Debug)]
struct GuitarPro6File<'file>(&'file [u8]);

#[derive(Debug)]
struct GuitarProFile<'file>(&'file [u8]);

#[derive(Debug)]
struct PowerTabFile<'file>(&'file [u8]);

#[derive(Debug)]
struct TuxGuitar1File<'file>(&'file [u8]);

#[derive(Debug)]
struct Cursor<'file> {
    tab_file: &'file [u8],
    position: usize,
}

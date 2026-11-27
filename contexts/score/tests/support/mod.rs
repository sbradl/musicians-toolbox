mod guitar_pro_6_and_7;
mod guitar_pro_bars;

use std::io::Write as _;

use rawzip::{CompressionMethod, ZipArchiveWriter};

pub use guitar_pro_6_and_7::*;

pub const E_STANDARD: [u8; 6] = [40, 45, 50, 55, 59, 64];
pub const DROP_D: [u8; 6] = [38, 45, 50, 55, 59, 64];
pub const BASS_E_STANDARD: [u8; 4] = [28, 33, 38, 43];
pub const B_STANDARD_7: [u8; 7] = [35, 40, 45, 50, 55, 59, 64];
pub const OPEN_G: [u8; 6] = [38, 43, 50, 55, 59, 62];

#[derive(Debug)]
pub struct GuitarProFile {
    major: u8,
    minor: u8,
    stated_version: Option<String>,
    stated_bars: Option<i32>,
    bars: usize,
    changes_key: bool,
    opening_tempo: i32,
    tempo_changes: Vec<TempoChange>,
    every_kind_of_beat: bool,
    name_fields_sized: Option<i32>,
    title: Vec<u8>,
    artist: String,
    album: String,
    tracks: Vec<TrackInFile>,
}

#[derive(Debug)]
struct TempoChange {
    bar: usize,
    beats_per_minute: i32,
}

#[derive(Debug)]
struct TrackInFile {
    lowest_string_first: Vec<u8>,
    is_drums: bool,
}

impl GuitarProFile {
    #[must_use]
    pub const fn version(major: u8, minor: u8) -> Self {
        Self {
            major,
            minor,
            stated_version: None,
            stated_bars: None,
            bars: 2,
            changes_key: false,
            opening_tempo: 120,
            tempo_changes: Vec::new(),
            every_kind_of_beat: false,
            name_fields_sized: None,
            title: Vec::new(),
            artist: String::new(),
            album: String::new(),
            tracks: Vec::new(),
        }
    }

    #[must_use]
    pub fn titled(self, title: &str) -> Self {
        self.titled_in_bytes(title.as_bytes())
    }

    #[must_use]
    pub fn titled_in_bytes(mut self, title: &[u8]) -> Self {
        title.clone_into(&mut self.title);

        self
    }

    #[must_use]
    pub fn stating_version(mut self, version: &str) -> Self {
        self.stated_version = Some(version.to_owned());

        self
    }

    #[must_use]
    pub const fn stating_bars(mut self, bars: i32) -> Self {
        self.stated_bars = Some(bars);

        self
    }

    #[must_use]
    pub const fn starting_at_tempo(mut self, beats_per_minute: i32) -> Self {
        self.opening_tempo = beats_per_minute;

        self
    }

    #[must_use]
    pub fn changing_tempo_in_bar(mut self, bar: usize, beats_per_minute: i32) -> Self {
        self.bars = self.bars.max(bar + 1);
        self.tempo_changes.push(TempoChange {
            bar,
            beats_per_minute,
        });

        self
    }

    #[must_use]
    pub const fn with_every_kind_of_beat(mut self) -> Self {
        self.every_kind_of_beat = true;

        self
    }

    #[must_use]
    pub const fn changing_key(mut self) -> Self {
        self.changes_key = true;

        self
    }

    #[must_use]
    pub const fn with_name_fields_sized(mut self, size: i32) -> Self {
        self.name_fields_sized = Some(size);

        self
    }

    #[must_use]
    pub fn by(mut self, artist: &str) -> Self {
        artist.clone_into(&mut self.artist);

        self
    }

    #[must_use]
    pub fn on_album(mut self, album: &str) -> Self {
        album.clone_into(&mut self.album);

        self
    }

    #[must_use]
    pub fn with_strings(mut self, lowest_string_first: &[u8]) -> Self {
        self.tracks.push(TrackInFile {
            lowest_string_first: lowest_string_first.to_vec(),
            is_drums: false,
        });

        self
    }

    #[must_use]
    pub fn with_drums(mut self) -> Self {
        self.tracks.push(TrackInFile {
            lowest_string_first: vec![0; 6],
            is_drums: true,
        });

        self
    }

    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        let mut file = Bytes::default();
        self.write_header(&mut file);
        file.zeros(64 * 12);

        if self.major == 5 {
            file.zeros(19 * 2 + 4);
        }
        match self.stated_bars {
            Some(bars) => file.number(bars),
            None => file.count(self.bars),
        }
        file.count(self.tracks.len());
        self.write_bar_headers(&mut file);
        self.write_tracks(&mut file);
        self.write_bars(&mut file);

        file.0
    }

    fn write_header(&self, file: &mut Bytes) {
        let version = format!("FICHIER GUITAR PRO v{}.{:02}", self.major, self.minor);
        file.fixed_size_text(self.stated_version.as_ref().unwrap_or(&version), 30);
        self.write_names(file);

        if self.major == 5 {
            file.text("");
        }
        file.texts(&["", "", ""]);
        file.number(2);
        file.texts(&["made up for the tests", "not a real song"]);
        self.write_header_after_notice(file);
    }

    fn write_names(&self, file: &mut Bytes) {
        if let Some(size) = self.name_fields_sized {
            for _ in 0..5 {
                file.number(size);
            }

            return;
        }
        file.text_in_bytes(&self.title);
        file.text("");
        file.text(&self.artist);
        file.text(&self.album);
        file.text("");
    }

    fn write_header_after_notice(&self, file: &mut Bytes) {
        match self.major {
            3 => {
                file.zeros(1);
                file.number(self.opening_tempo);
                file.zeros(4);
            }
            4 => {
                file.zeros(1);
                file.lyrics();
                file.number(self.opening_tempo);
                file.zeros(4 + 1);
            }
            _ => self.write_gp5_page_setup(file),
        }
    }

    fn write_gp5_page_setup(&self, file: &mut Bytes) {
        file.lyrics();

        if self.minor > 0 {
            file.zeros(4 + 4 + 11);
        }
        file.zeros(2 * 4 + 4 * 4 + 4 + 2);
        file.texts(&["", "", "", "", "", "", "", "", "", ""]);
        file.text("Moderato");
        file.number(self.opening_tempo);

        if self.minor > 0 {
            file.zeros(1);
        }
        file.zeros(1 + 4);
    }

    fn write_bar_headers(&self, file: &mut Bytes) {
        for bar in 0..self.bars {
            if self.major == 5 && bar > 0 {
                file.zeros(1);
            }

            if bar == 0 {
                self.write_four_four_bar_header(file);
            } else {
                self.write_marker_bar_header(file);
            }
        }
    }

    fn write_four_four_bar_header(&self, file: &mut Bytes) {
        if self.changes_key {
            file.0.extend([0x43, 4, 4, 2, 0]);
        } else {
            file.0.extend([0x03, 4, 4]);
        }

        if self.major == 5 {
            file.zeros(4 + 1 + 1);
        }
    }

    fn write_marker_bar_header(&self, file: &mut Bytes) {
        file.0.push(0x20);
        file.text("Verse");
        file.zeros(4);

        if self.major == 5 {
            file.zeros(1 + 1);
        }
    }

    fn write_tracks(&self, file: &mut Bytes) {
        for (index, track) in self.tracks.iter().enumerate() {
            if self.major == 5 && (index == 0 || self.minor == 0) {
                file.zeros(1);
            }
            file.0.push(u8::from(track.is_drums));
            file.fixed_size_text("Invented Track", 40);
            write_strings(file, track);

            if self.major == 5 {
                self.write_gp5_track_sound(file);
            }
        }
    }

    fn write_gp5_track_sound(&self, file: &mut Bytes) {
        file.zeros(2 + 1 + 1 + 1 + 12 + 12 + 4 + 4 + 4);

        if self.minor == 0 {
            file.zeros(2 + 1);
        } else {
            file.zeros(4 + 4);
            file.texts(&["", ""]);
        }
    }
}

fn write_strings(file: &mut Bytes, track: &TrackInFile) {
    file.count(track.lowest_string_first.len());
    let mut highest_first = Vec::new();
    for &pitch in track.lowest_string_first.iter().rev() {
        highest_first.push(i32::from(pitch));
    }
    highest_first.resize(7, UNUSED_STRING);
    for pitch in highest_first {
        file.number(pitch);
    }
    let channel = if track.is_drums { 10 } else { 1 };
    file.number(1);
    file.number(channel);
    file.zeros(4 * 4);
}

const UNUSED_STRING: i32 = -1;

#[derive(Debug, Default)]
struct Bytes(Vec<u8>);

impl Bytes {
    fn zeros(&mut self, count: usize) {
        self.0.resize(self.0.len() + count, 0);
    }

    fn number(&mut self, number: i32) {
        self.0.extend(number.to_le_bytes());
    }

    fn count(&mut self, count: usize) {
        self.number(i32::try_from(count).unwrap_or(i32::MAX));
    }

    fn fixed_size_text(&mut self, text: &str, size: usize) {
        self.fixed_size_text_in_bytes(text.as_bytes(), size);
    }

    fn fixed_size_text_in_bytes(&mut self, text: &[u8], size: usize) {
        self.0.push(u8::try_from(text.len()).unwrap_or(u8::MAX));
        let mut field = text.to_vec();
        field.resize(size, 0);
        self.0.extend(field);
    }

    fn text(&mut self, text: &str) {
        self.text_in_bytes(text.as_bytes());
    }

    fn text_in_bytes(&mut self, text: &[u8]) {
        self.count(text.len() + 1);
        self.fixed_size_text_in_bytes(text, text.len());
    }

    fn texts(&mut self, texts: &[&str]) {
        for text in texts {
            self.text(text);
        }
    }

    fn lyrics(&mut self) {
        self.zeros(4);
        for _ in 0..5 {
            self.zeros(4);
            self.count(0);
        }
    }
}

#[must_use]
pub fn zip_holding(files: &[(&str, &[u8])]) -> Vec<u8> {
    zip_of(files, CompressionMethod::DEFLATE).unwrap_or_default()
}

#[must_use]
pub fn zip_storing(files: &[(&str, &[u8])]) -> Vec<u8> {
    zip_of(files, CompressionMethod::STORE).unwrap_or_default()
}

#[must_use]
pub fn zip_shrinking(files: &[(&str, &[u8])]) -> Vec<u8> {
    zip_of(files, CompressionMethod::SHRUNK).unwrap_or_default()
}

#[must_use]
pub fn with_local_header_broken(mut zip: Vec<u8>, name: &str) -> Vec<u8> {
    let local_header_size = 30;
    let name_at = zip
        .windows(name.len())
        .position(|window| window == name.as_bytes())
        .unwrap_or(local_header_size);
    zip[name_at - local_header_size] = 0;

    zip
}

fn zip_of(
    files: &[(&str, &[u8])],
    compression: CompressionMethod,
) -> Result<Vec<u8>, rawzip::Error> {
    let mut zip = Vec::new();
    let mut archive = ZipArchiveWriter::new(&mut zip);
    for (name, contents) in files {
        add_entry(&mut archive, name, contents, compression)?;
    }
    _ = archive.finish()?;

    Ok(zip)
}

fn add_entry(
    archive: &mut ZipArchiveWriter<&mut Vec<u8>>,
    name: &str,
    contents: &[u8],
    compression: CompressionMethod,
) -> Result<(), rawzip::Error> {
    let (mut entry, config) = archive
        .new_file(name)
        .compression_method(compression)
        .start()?;
    let mut uncompressed = config.wrap(Vec::new());
    uncompressed.write_all(contents)?;
    let (_, descriptor) = uncompressed.finish()?;
    entry.write_all(&packed(contents, compression))?;
    _ = entry.finish(descriptor)?;

    Ok(())
}

fn packed(contents: &[u8], compression: CompressionMethod) -> Vec<u8> {
    if compression == CompressionMethod::DEFLATE {
        return miniz_oxide::deflate::compress_to_vec(contents, 6);
    }

    contents.to_vec()
}

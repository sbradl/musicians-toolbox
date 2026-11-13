use std::io::Write as _;

use rawzip::{CompressionMethod, ZipArchiveWriter};

pub const E_STANDARD: [u8; 6] = [40, 45, 50, 55, 59, 64];
pub const DROP_D: [u8; 6] = [38, 45, 50, 55, 59, 64];
pub const BASS_E_STANDARD: [u8; 4] = [28, 33, 38, 43];
pub const OPEN_G: [u8; 6] = [38, 43, 50, 55, 59, 62];

#[derive(Debug)]
pub struct GuitarProFile {
    major: u8,
    minor: u8,
    bars: usize,
    tracks: Vec<TrackInFile>,
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
            bars: 2,
            tracks: Vec::new(),
        }
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
        file.count(self.bars);
        file.count(self.tracks.len());
        self.write_bar_headers(&mut file);
        self.write_tracks(&mut file);
        file.zeros(64);

        file.0
    }

    fn write_header(&self, file: &mut Bytes) {
        file.fixed_size_text(
            &format!("FICHIER GUITAR PRO v{}.{:02}", self.major, self.minor),
            30,
        );
        file.text("Invented Title");
        file.text("");
        file.text("Invented Artist");
        file.text("Invented Album");
        file.text("");

        if self.major == 5 {
            file.text("");
        }
        file.texts(&["", "", ""]);
        file.number(2);
        file.texts(&["made up for the tests", "not a real song"]);
        self.write_header_after_notice(file);
    }

    fn write_header_after_notice(&self, file: &mut Bytes) {
        match self.major {
            3 => file.zeros(1 + 4 + 4),
            4 => {
                file.zeros(1);
                file.lyrics();
                file.zeros(4 + 4 + 1);
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
        file.number(120);

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
        file.0.extend([0x03, 4, 4]);

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
        self.0.push(u8::try_from(text.len()).unwrap_or(u8::MAX));
        let mut field = text.as_bytes().to_vec();
        field.resize(size, 0);
        self.0.extend(field);
    }

    fn text(&mut self, text: &str) {
        self.count(text.len() + 1);
        self.fixed_size_text(text, text.len());
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
    zip_of(files).unwrap_or_default()
}

fn zip_of(files: &[(&str, &[u8])]) -> Result<Vec<u8>, rawzip::Error> {
    let mut zip = Vec::new();
    let mut archive = ZipArchiveWriter::new(&mut zip);
    for (name, contents) in files {
        add_deflated(&mut archive, name, contents)?;
    }
    _ = archive.finish()?;

    Ok(zip)
}

fn add_deflated(
    archive: &mut ZipArchiveWriter<&mut Vec<u8>>,
    name: &str,
    contents: &[u8],
) -> Result<(), rawzip::Error> {
    let (mut entry, config) = archive
        .new_file(name)
        .compression_method(CompressionMethod::DEFLATE)
        .start()?;
    let mut uncompressed = config.wrap(Vec::new());
    uncompressed.write_all(contents)?;
    let (_, descriptor) = uncompressed.finish()?;
    entry.write_all(&miniz_oxide::deflate::compress_to_vec(contents, 6))?;
    _ = entry.finish(descriptor)?;

    Ok(())
}

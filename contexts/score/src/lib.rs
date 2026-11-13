mod cursor;
mod guitar_pro;
mod zip;

use mt_music_kernel::Tuning;

pub const LARGEST_TAB_FILE: usize = 64 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub struct ScoreHeader {
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
        return read_unzipped_header(&zip.tab_file()?);
    }

    read_unzipped_header(tab_file)
}

fn read_unzipped_header(tab_file: &[u8]) -> Result<ScoreHeader, Unreadable> {
    let Some(guitar_pro) = GuitarProFile::recognized(tab_file) else {
        return Err(Unreadable);
    };

    guitar_pro.read_header()
}

#[derive(Debug)]
struct Zip<'file>(&'file [u8]);

#[derive(Debug)]
struct GuitarProFile<'file>(&'file [u8]);

#[derive(Debug)]
struct Cursor<'file> {
    tab_file: &'file [u8],
    position: usize,
}

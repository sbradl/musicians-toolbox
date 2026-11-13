use std::io::{self, BufReader, Read};

use miniz_oxide::inflate::decompress_to_vec_with_limit;
use miniz_oxide::inflate::stream::{InflateState, inflate};
use miniz_oxide::{DataFormat, MZFlush};
use rawzip::{CompressionMethod, ZipArchive, ZipFileHeaderRecord, ZipSliceArchive};

use crate::{
    LARGEST_SCORE_XML, LARGEST_TAB_FILE, ScoreHeader, ScoreXml, Unreadable, Zip,
    read_unzipped_header,
};

const SCORE_XML_PATH: &[u8] = b"Content/score.gpif";
const MOST_ENTRIES: usize = 4096;
const TAB_FILE_EXTENSIONS: [&[u8]; 8] =
    [b"gp3", b"gp4", b"gp5", b"gpx", b"gp", b"gtp", b"tg", b"ptb"];

impl<'file> Zip<'file> {
    pub(super) fn recognized(tab_file: &'file [u8]) -> Option<Self> {
        if tab_file.starts_with(b"PK\x03\x04") {
            return Some(Self(tab_file));
        }

        None
    }

    pub(super) fn read_header(&self) -> Result<ScoreHeader, Unreadable> {
        let Ok(archive) = ZipArchive::from_slice(self.0) else {
            return Err(Unreadable);
        };
        let mut entries = archive.entries();
        for _ in 0..MOST_ENTRIES {
            let Ok(Some(entry)) = entries.next_entry() else {
                return Err(Unreadable);
            };

            if entry.file_path().as_bytes() == SCORE_XML_PATH {
                return read_score_xml_header(&archive, &entry);
            }

            if is_tab_file(&entry) {
                return read_unzipped_header(&unpacked(&archive, &entry)?);
            }
        }

        Err(Unreadable)
    }
}

fn is_tab_file(entry: &ZipFileHeaderRecord<'_>) -> bool {
    let name = entry.file_path().as_bytes().to_ascii_lowercase();
    let Some(dot) = name.iter().rposition(|&byte| byte == b'.') else {
        return false;
    };

    !entry.is_dir() && TAB_FILE_EXTENSIONS.contains(&&name[dot + 1..])
}

fn read_score_xml_header(
    archive: &ZipSliceArchive<&[u8]>,
    entry: &ZipFileHeaderRecord<'_>,
) -> Result<ScoreHeader, Unreadable> {
    let Ok(found) = archive.get_entry(entry.wayfinder()) else {
        return Err(Unreadable);
    };
    let data = found.data();

    match entry.compression_method() {
        CompressionMethod::STORE => ScoreXml(data).read_header(),
        CompressionMethod::DEFLATE => ScoreXml(BufReader::new(Inflating::over(data))).read_header(),
        _ => Err(Unreadable),
    }
}

fn unpacked(
    archive: &ZipSliceArchive<&[u8]>,
    entry: &ZipFileHeaderRecord<'_>,
) -> Result<Vec<u8>, Unreadable> {
    let Ok(found) = archive.get_entry(entry.wayfinder()) else {
        return Err(Unreadable);
    };
    let data = found.data();
    let compression = entry.compression_method();

    if compression == CompressionMethod::STORE && data.len() <= LARGEST_TAB_FILE {
        return Ok(data.to_vec());
    }

    if compression != CompressionMethod::DEFLATE {
        return Err(Unreadable);
    }

    decompress_to_vec_with_limit(data, LARGEST_TAB_FILE).map_err(|_| Unreadable)
}

struct Inflating<'file> {
    state: Box<InflateState>,
    deflated: &'file [u8],
    inflated: usize,
}

impl<'file> Inflating<'file> {
    fn over(deflated: &'file [u8]) -> Self {
        Self {
            state: InflateState::new_boxed(DataFormat::Raw),
            deflated,
            inflated: 0,
        }
    }
}

impl Read for Inflating<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let result = inflate(&mut self.state, self.deflated, buffer, MZFlush::None);
        self.deflated = &self.deflated[result.bytes_consumed..];
        self.inflated += result.bytes_written;

        if result.status.is_err() && result.bytes_written == 0 || self.inflated > LARGEST_SCORE_XML
        {
            return Err(io::ErrorKind::InvalidData.into());
        }

        Ok(result.bytes_written)
    }
}

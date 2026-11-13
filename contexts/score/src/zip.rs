use miniz_oxide::inflate::decompress_to_vec_with_limit;
use rawzip::{CompressionMethod, ZipArchive, ZipFileHeaderRecord, ZipSliceArchive};

use crate::{LARGEST_TAB_FILE, Unreadable, Zip};

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

    pub(super) fn tab_file(&self) -> Result<Vec<u8>, Unreadable> {
        let Ok(archive) = ZipArchive::from_slice(self.0) else {
            return Err(Unreadable);
        };
        let mut entries = archive.entries();
        for _ in 0..MOST_ENTRIES {
            let Ok(Some(entry)) = entries.next_entry() else {
                return Err(Unreadable);
            };

            if is_tab_file(&entry) {
                return unpacked(&archive, &entry);
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

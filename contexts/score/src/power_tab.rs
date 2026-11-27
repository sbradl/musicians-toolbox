use crate::{Cursor, PowerTabFile, ScoreHeader, Unreadable, decoded, stated};

const MAGIC: &[u8] = b"ptab";
const FIRST_VERSION_READ: u16 = 3;
const LESSON: u8 = 1;
const PUBLIC_AUDIO_RELEASE: u8 = 0;
const PUBLIC_VIDEO_RELEASE: u8 = 1;
const BOOTLEG_RELEASE: u8 = 2;
const LONGER_NAME: u8 = 0xFF;
const UNICODE_NAME: u16 = 0xFFFE;
const LONGEST_NAME: u16 = 0xFFFF;

impl<'file> PowerTabFile<'file> {
    pub(super) fn recognized(tab_file: &'file [u8]) -> Option<Self> {
        if tab_file.starts_with(MAGIC) {
            return Some(Self(tab_file));
        }

        None
    }

    pub(super) fn read_header(&self) -> Result<ScoreHeader, Unreadable> {
        let mut cursor = Cursor::over(self.0);
        cursor.skip(MAGIC.len())?;

        if u16::from_le_bytes(cursor.array()?) < FIRST_VERSION_READ {
            return Err(Unreadable);
        }

        if cursor.byte()? == LESSON {
            let title = read_name(&mut cursor)?;

            return Ok(header_naming([title, None, None]));
        }

        Ok(header_naming(read_song_names(&mut cursor)?))
    }
}

fn read_song_names(cursor: &mut Cursor<'_>) -> Result<[Option<String>; 3], Unreadable> {
    let content_type = 1;
    cursor.skip(content_type)?;
    let title = read_name(cursor)?;
    let artist = read_name(cursor)?;
    let album = read_album(cursor)?;

    Ok([title, artist, album])
}

fn read_album(cursor: &mut Cursor<'_>) -> Result<Option<String>, Unreadable> {
    let album_type = 1;

    match cursor.byte()? {
        PUBLIC_AUDIO_RELEASE => {
            cursor.skip(album_type)?;

            read_name(cursor)
        }
        PUBLIC_VIDEO_RELEASE | BOOTLEG_RELEASE => read_name(cursor),
        _ => Ok(None),
    }
}

fn read_name(cursor: &mut Cursor<'_>) -> Result<Option<String>, Unreadable> {
    let length = read_name_length(cursor)?;

    Ok(stated(&decoded(cursor.bytes(length)?)))
}

fn read_name_length(cursor: &mut Cursor<'_>) -> Result<usize, Unreadable> {
    let short_length = cursor.byte()?;

    if short_length != LONGER_NAME {
        return Ok(usize::from(short_length));
    }

    match u16::from_le_bytes(cursor.array()?) {
        UNICODE_NAME => Err(Unreadable),
        LONGEST_NAME => cursor.count_up_to(usize::MAX),
        length => Ok(usize::from(length)),
    }
}

fn header_naming([title, artist, album]: [Option<String>; 3]) -> ScoreHeader {
    ScoreHeader {
        title,
        artist,
        album,
        tracks: Vec::new(),
        written_tempos: Vec::new(),
    }
}

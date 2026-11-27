use crate::{Cursor, ScoreHeader, TuxGuitar1File, Unreadable, stated};

const TUX_GUITAR_1_FORMAT: &str = "TuxGuitar File Format";

impl<'file> TuxGuitar1File<'file> {
    pub(super) fn recognized(tab_file: &'file [u8]) -> Option<Self> {
        let format = read_utf16_text(&mut Cursor::over(tab_file)).ok()?;

        if format.starts_with(TUX_GUITAR_1_FORMAT) {
            return Some(Self(tab_file));
        }

        None
    }

    pub(super) fn read_header(&self) -> Result<ScoreHeader, Unreadable> {
        let mut cursor = Cursor::over(self.0);
        let _format = read_utf16_text(&mut cursor)?;
        let title = read_utf16_text(&mut cursor)?;
        let artist = read_utf16_text(&mut cursor)?;
        let album = read_utf16_text(&mut cursor)?;

        Ok(ScoreHeader {
            title: stated(&title),
            artist: stated(&artist),
            album: stated(&album),
            tracks: Vec::new(),
            written_tempos: Vec::new(),
        })
    }
}

fn read_utf16_text(cursor: &mut Cursor<'_>) -> Result<String, Unreadable> {
    let units = usize::from(cursor.byte()?);
    let mut big_endian_units = Vec::new();
    for _ in 0..units {
        big_endian_units.push(u16::from_be_bytes(cursor.array()?));
    }
    let mut text = String::new();
    for character in char::decode_utf16(big_endian_units) {
        text.push(character.unwrap_or(char::REPLACEMENT_CHARACTER));
    }

    Ok(text)
}

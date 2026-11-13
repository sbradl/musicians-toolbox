use mt_music_kernel::{Pitch, Tuning};

use crate::{Cursor, GuitarProFile, ScoreHeader, Track, Unreadable};

const MOST_BARS: usize = 10_000;
const MOST_TRACKS: usize = 128;
const STRING_SLOTS: usize = 7;
const DRUM_CHANNEL: i32 = 10;
const HIGHEST_MIDI_NUMBER: u8 = 127;

const NUMBER: usize = 4;
const VERSION_FIELD: usize = 30;
const TRACK_NAME_FIELD: usize = 1 + 40;
const MIDI_CHANNELS: usize = 64 * (NUMBER + 8);
const LYRIC_LINES: usize = 5;
const GP5_PAGE_SETUP: usize = 2 * NUMBER + 4 * NUMBER + NUMBER + 2;
const GP5_HEADER_AND_FOOTER_TEXTS: usize = 10;
const GP5_MASTER_EFFECT: usize = NUMBER + NUMBER + 11;
const GP5_DIRECTIONS_AND_MASTER_REVERB: usize = 19 * 2 + NUMBER;
const GP5_TRACK_SOUND: usize = 2 + 1 + 1 + 1 + 12 + 12 + NUMBER + NUMBER + NUMBER;

const BAR_HAS_NUMERATOR: u8 = 0x01;
const BAR_HAS_DENOMINATOR: u8 = 0x02;
const BAR_HAS_TIME_SIGNATURE: u8 = BAR_HAS_NUMERATOR | BAR_HAS_DENOMINATOR;
const BAR_CLOSES_REPEAT: u8 = 0x08;
const BAR_HAS_ALTERNATE_ENDING: u8 = 0x10;
const BAR_HAS_MARKER: u8 = 0x20;
const BAR_HAS_KEY_SIGNATURE: u8 = 0x40;
const TRACK_IS_DRUMS: u8 = 0x01;

#[derive(Clone, Copy, Debug)]
struct Version {
    major: u8,
    minor: u8,
}

impl Version {
    const fn is_5(self) -> bool {
        self.major == 5
    }

    const fn is_5_after_5_00(self) -> bool {
        self.is_5() && self.minor > 0
    }
}

#[derive(Clone, Copy, Debug)]
struct BarFlags(u8);

impl BarFlags {
    const fn has(self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}

impl<'file> GuitarProFile<'file> {
    pub(super) fn recognized(tab_file: &'file [u8]) -> Option<Self> {
        let version_text = tab_file.get(1..=VERSION_FIELD)?;

        if version_text.windows(6).any(|word| word == b"GUITAR") {
            return Some(Self(tab_file));
        }

        None
    }

    pub(super) fn read_header(&self) -> Result<ScoreHeader, Unreadable> {
        let mut cursor = Cursor::over(self.0);
        let version = read_version(&mut cursor)?;
        skip_song_information(&mut cursor, version)?;
        skip_page_setup_and_tempo(&mut cursor, version)?;
        skip_midi_channels(&mut cursor, version)?;
        let bars = cursor.count_up_to(MOST_BARS)?;
        let tracks = cursor.count_up_to(MOST_TRACKS)?;

        if tracks == 0 {
            return Err(Unreadable);
        }
        skip_bar_headers(&mut cursor, version, bars)?;

        Ok(ScoreHeader {
            tracks: read_tracks(&mut cursor, version, tracks)?,
        })
    }
}

fn read_version(cursor: &mut Cursor<'_>) -> Result<Version, Unreadable> {
    let length = usize::from(cursor.byte()?);
    let field = cursor.bytes(VERSION_FIELD)?;
    let text = String::from_utf8_lossy(&field[..length.min(VERSION_FIELD)]);
    let Some((_, number)) = text.rsplit_once('v') else {
        return Err(Unreadable);
    };
    let Some((major, minor)) = number.split_once('.') else {
        return Err(Unreadable);
    };
    let (Ok(major), Ok(minor)) = (major.parse(), minor.trim().parse()) else {
        return Err(Unreadable);
    };

    if !(3..=5).contains(&major) {
        return Err(Unreadable);
    }

    Ok(Version { major, minor })
}

fn skip_song_information(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let title_to_instructions = if version.is_5() { 9 } else { 8 };
    skip_texts(cursor, title_to_instructions)?;
    let notice_lines = cursor.count_up_to(usize::MAX)?;

    skip_texts(cursor, notice_lines)
}

fn skip_page_setup_and_tempo(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let triplet_feel = 1;
    let tempo_and_key = NUMBER + NUMBER;
    let octave = 1;

    match version.major {
        3 => cursor.skip(triplet_feel + tempo_and_key),
        4 => {
            cursor.skip(triplet_feel)?;
            skip_lyrics(cursor)?;

            cursor.skip(tempo_and_key + octave)
        }
        _ => skip_gp5_page_setup_and_tempo(cursor, version),
    }
}

fn skip_gp5_page_setup_and_tempo(
    cursor: &mut Cursor<'_>,
    version: Version,
) -> Result<(), Unreadable> {
    let tempo_name = 1;
    let hide_tempo = 1;
    let key_and_octave = 1 + NUMBER;
    skip_lyrics(cursor)?;

    if version.is_5_after_5_00() {
        cursor.skip(GP5_MASTER_EFFECT)?;
    }
    cursor.skip(GP5_PAGE_SETUP)?;
    skip_texts(cursor, GP5_HEADER_AND_FOOTER_TEXTS + tempo_name)?;
    cursor.skip(NUMBER)?;

    if version.is_5_after_5_00() {
        cursor.skip(hide_tempo)?;
    }

    cursor.skip(key_and_octave)
}

fn skip_lyrics(cursor: &mut Cursor<'_>) -> Result<(), Unreadable> {
    let track = NUMBER;
    let starting_bar = NUMBER;
    cursor.skip(track)?;
    for _ in 0..LYRIC_LINES {
        cursor.skip(starting_bar)?;
        cursor.skip_counted_bytes()?;
    }

    Ok(())
}

fn skip_midi_channels(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    cursor.skip(MIDI_CHANNELS)?;

    if version.is_5() {
        return cursor.skip(GP5_DIRECTIONS_AND_MASTER_REVERB);
    }

    Ok(())
}

fn skip_bar_headers(
    cursor: &mut Cursor<'_>,
    version: Version,
    bars: usize,
) -> Result<(), Unreadable> {
    let gp5_separator = 1;
    for bar in 0..bars {
        if version.is_5() && bar > 0 {
            cursor.skip(gp5_separator)?;
        }
        skip_bar_header(cursor, version)?;
    }

    Ok(())
}

fn skip_bar_header(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let flags = BarFlags(cursor.byte()?);
    skip_bar_numbers_before_marker(cursor, flags, version)?;

    if flags.has(BAR_HAS_MARKER) {
        let color = NUMBER;
        skip_texts(cursor, 1)?;
        cursor.skip(color)?;
    }

    if flags.has(BAR_HAS_KEY_SIGNATURE) {
        cursor.skip(2)?;
    }

    if version.is_5() {
        return skip_gp5_bar_header_end(cursor, flags);
    }

    Ok(())
}

fn skip_bar_numbers_before_marker(
    cursor: &mut Cursor<'_>,
    flags: BarFlags,
    version: Version,
) -> Result<(), Unreadable> {
    let gp3_or_4_ending = flags.has(BAR_HAS_ALTERNATE_ENDING) && !version.is_5();
    let one_byte_each = [
        flags.has(BAR_HAS_NUMERATOR),
        flags.has(BAR_HAS_DENOMINATOR),
        flags.has(BAR_CLOSES_REPEAT),
        gp3_or_4_ending,
    ];
    let mut bytes = 0;
    for present in one_byte_each {
        bytes += usize::from(present);
    }

    cursor.skip(bytes)
}

fn skip_gp5_bar_header_end(cursor: &mut Cursor<'_>, flags: BarFlags) -> Result<(), Unreadable> {
    let has_alternate_endings = flags.has(BAR_HAS_ALTERNATE_ENDING);
    let beam_groups = NUMBER;
    let triplet_feel = 1;
    cursor.skip(usize::from(has_alternate_endings))?;

    if flags.has(BAR_HAS_TIME_SIGNATURE) {
        cursor.skip(beam_groups)?;
    }
    cursor.skip(usize::from(!has_alternate_endings))?;

    cursor.skip(triplet_feel)
}

fn read_tracks(
    cursor: &mut Cursor<'_>,
    version: Version,
    tracks: usize,
) -> Result<Vec<Track>, Unreadable> {
    let gp5_separator = 1;
    let mut read = Vec::new();
    for index in 0..tracks {
        if version.is_5() && (index == 0 || version.minor == 0) {
            cursor.skip(gp5_separator)?;
        }
        read.push(read_track(cursor, version)?);
    }

    Ok(read)
}

fn read_track(cursor: &mut Cursor<'_>, version: Version) -> Result<Track, Unreadable> {
    let flags = cursor.byte()?;
    cursor.skip(TRACK_NAME_FIELD)?;
    let strings = cursor.count_up_to(STRING_SLOTS)?;
    let highest_first = read_string_slots(cursor)?;
    let port = NUMBER;
    let effect_channel_frets_capo_and_color = 4 * NUMBER;
    cursor.skip(port)?;
    let channel = cursor.number()?;
    cursor.skip(effect_channel_frets_capo_and_color)?;
    skip_track_sound(cursor, version)?;

    if flags & TRACK_IS_DRUMS != 0 || channel == DRUM_CHANNEL {
        return Ok(Track { tuning: None });
    }

    Ok(Track {
        tuning: Some(tuning_of(&highest_first, strings)?),
    })
}

fn read_string_slots(cursor: &mut Cursor<'_>) -> Result<Vec<i32>, Unreadable> {
    let mut slots = Vec::new();
    for _ in 0..STRING_SLOTS {
        slots.push(cursor.number()?);
    }

    Ok(slots)
}

fn tuning_of(highest_first: &[i32], strings: usize) -> Result<Tuning, Unreadable> {
    if strings == 0 {
        return Err(Unreadable);
    }
    let mut lowest_string_first = Vec::new();
    for &midi_number in highest_first[..strings].iter().rev() {
        lowest_string_first.push(pitch_of(midi_number)?);
    }

    Ok(Tuning::of(lowest_string_first))
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

fn skip_track_sound(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    if !version.is_5() {
        return Ok(());
    }
    cursor.skip(GP5_TRACK_SOUND)?;

    if version.minor == 0 {
        return cursor.skip(2 + 1);
    }
    let effect_number_and_equalizer = NUMBER + NUMBER;
    cursor.skip(effect_number_and_equalizer)?;

    skip_texts(cursor, 2)
}

fn skip_texts(cursor: &mut Cursor<'_>, texts: usize) -> Result<(), Unreadable> {
    for _ in 0..texts {
        let Ok(size) = usize::try_from(cursor.number()?) else {
            continue;
        };
        cursor.skip(size)?;
    }

    Ok(())
}

use mt_music_kernel::WrittenTempo;

use super::{Flags, NUMBER, Version, skip_texts};
use crate::{Cursor, Unreadable, stated_tempo};

const BEAT_HAS_CHORD: u8 = 0x02;
const BEAT_HAS_TEXT: u8 = 0x04;
const BEAT_HAS_EFFECTS: u8 = 0x08;
const BEAT_HAS_MIX_TABLE: u8 = 0x10;
const BEAT_HAS_TUPLET: u8 = 0x20;
const BEAT_HAS_STATUS: u8 = 0x40;
const SEVEN_STRINGS: u8 = 0x7F;
const CHORD_IS_NEW_FORMAT: u8 = 0x01;
const GP3_NEW_CHORD: usize = 124;
const GP4_AND_5_NEW_CHORD: usize = 106;
const STRINGS_OF_OLD_CHORD: usize = 6;
const GP5_BEAT_BREAKS_SECONDARY_BEAMS: u8 = 0x08;

const GP3_BEAT_HAS_TAPPING_OR_TREMOLO_BAR: u8 = 0x20;
const BEAT_HAS_STROKE: u8 = 0x40;
const BEAT_HAS_TAPPING: u8 = 0x20;
const BEAT_HAS_TREMOLO_BAR: u8 = 0x04;
const BEAT_HAS_PICK_STROKE: u8 = 0x02;

const NOTE_HAS_DURATION_OR_PERCENT: u8 = 0x01;
const NOTE_HAS_EFFECTS: u8 = 0x08;
const NOTE_HAS_VELOCITY: u8 = 0x10;
const NOTE_HAS_TYPE_AND_FRET: u8 = 0x20;
const NOTE_HAS_FINGERING: u8 = 0x80;
const NOTE_HAS_BEND: u8 = 0x01;
const NOTE_HAS_GRACE: u8 = 0x10;
const NOTE_HAS_TREMOLO_PICKING: u8 = 0x04;
const NOTE_HAS_SLIDE: u8 = 0x08;
const NOTE_HAS_HARMONIC: u8 = 0x10;
const NOTE_HAS_TRILL: u8 = 0x20;
const GP5_ARTIFICIAL_HARMONIC: u8 = 2;
const GP5_TAPPED_HARMONIC: u8 = 3;

const MIX_TABLE_VALUES: usize = 6;
const GP5_RSE_INSTRUMENT: usize = 16;
const BEND_POINT: usize = 4 + 4 + 1;

pub(super) fn tempo_changes(
    cursor: &mut Cursor<'_>,
    version: Version,
    bar_tracks: usize,
) -> Result<Vec<WrittenTempo>, Unreadable> {
    let voices = if version.is_5() { 2 } else { 1 };
    skip_gp5_track_end(cursor, version)?;
    let mut changes = Vec::new();
    for _ in 0..bar_tracks {
        for _ in 0..voices {
            changes.extend(read_voice(cursor, version)?);
        }
        skip_gp5_line_break(cursor, version);
    }

    Ok(changes)
}

fn skip_gp5_track_end(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let gp5_00_track_end = 2;
    let gp5_track_end = 1;

    match (version.is_5(), version.minor) {
        (false, _) => Ok(()),
        (true, 0) => cursor.skip(gp5_00_track_end),
        (true, _) => cursor.skip(gp5_track_end),
    }
}

fn skip_gp5_line_break(cursor: &mut Cursor<'_>, version: Version) {
    let line_break = 1;

    if version.is_5() {
        _ = cursor.skip(line_break);
    }
}

fn read_voice(cursor: &mut Cursor<'_>, version: Version) -> Result<Vec<WrittenTempo>, Unreadable> {
    let beats = cursor.count_up_to(usize::MAX)?;
    let mut changes = Vec::new();
    for _ in 0..beats {
        if let Some(change) = read_beat(cursor, version)? {
            changes.push(change);
        }
    }

    Ok(changes)
}

fn read_beat(
    cursor: &mut Cursor<'_>,
    version: Version,
) -> Result<Option<WrittenTempo>, Unreadable> {
    let flags = Flags(cursor.byte()?);
    skip_beat_before_mix_table(cursor, version, flags)?;
    let change = read_mix_table_if_any(cursor, version, flags)?;
    skip_notes(cursor, version)?;
    skip_gp5_beat_end(cursor, version)?;

    Ok(change)
}

fn skip_beat_before_mix_table(
    cursor: &mut Cursor<'_>,
    version: Version,
    flags: Flags,
) -> Result<(), Unreadable> {
    let status = usize::from(flags.has(BEAT_HAS_STATUS));
    let duration = 1;
    let tuplet = NUMBER * usize::from(flags.has(BEAT_HAS_TUPLET));
    cursor.skip(status + duration + tuplet)?;

    if flags.has(BEAT_HAS_CHORD) {
        skip_chord(cursor, version)?;
    }

    if flags.has(BEAT_HAS_TEXT) {
        skip_texts(cursor, 1)?;
    }

    if flags.has(BEAT_HAS_EFFECTS) {
        skip_beat_effects(cursor, version)?;
    }

    Ok(())
}

fn skip_chord(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    if Flags(cursor.byte()?).has(CHORD_IS_NEW_FORMAT) {
        let new_chord = if version.major == 3 {
            GP3_NEW_CHORD
        } else {
            GP4_AND_5_NEW_CHORD
        };

        return cursor.skip(new_chord);
    }
    skip_texts(cursor, 1)?;

    if cursor.number()? != 0 {
        cursor.skip(STRINGS_OF_OLD_CHORD * NUMBER)?;
    }

    Ok(())
}

fn skip_beat_effects(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let stroke = 2;

    if version.major == 3 {
        let flags = Flags(cursor.byte()?);
        let tapping_and_tremolo_bar = 1 + NUMBER;

        if flags.has(GP3_BEAT_HAS_TAPPING_OR_TREMOLO_BAR) {
            cursor.skip(tapping_and_tremolo_bar)?;
        }

        if flags.has(BEAT_HAS_STROKE) {
            cursor.skip(stroke)?;
        }

        return Ok(());
    }

    skip_gp4_and_5_beat_effects(cursor)
}

fn skip_gp4_and_5_beat_effects(cursor: &mut Cursor<'_>) -> Result<(), Unreadable> {
    let first = Flags(cursor.byte()?);
    let second = Flags(cursor.byte()?);
    let tapping = 1;
    let stroke = 2;
    let pick_stroke = 1;
    cursor.skip(tapping * usize::from(first.has(BEAT_HAS_TAPPING)))?;

    if second.has(BEAT_HAS_TREMOLO_BAR) {
        skip_bend(cursor)?;
    }
    cursor.skip(stroke * usize::from(first.has(BEAT_HAS_STROKE)))?;

    cursor.skip(pick_stroke * usize::from(second.has(BEAT_HAS_PICK_STROKE)))
}

fn read_mix_table_if_any(
    cursor: &mut Cursor<'_>,
    version: Version,
    flags: Flags,
) -> Result<Option<WrittenTempo>, Unreadable> {
    if flags.has(BEAT_HAS_MIX_TABLE) {
        return read_mix_table(cursor, version);
    }

    Ok(None)
}

fn read_mix_table(
    cursor: &mut Cursor<'_>,
    version: Version,
) -> Result<Option<WrittenTempo>, Unreadable> {
    let instrument = 1;
    let gp5_rse_instrument = GP5_RSE_INSTRUMENT * usize::from(version.is_5());
    cursor.skip(instrument + gp5_rse_instrument)?;
    let values = cursor.bytes(MIX_TABLE_VALUES)?;

    if version.is_5() {
        skip_texts(cursor, 1)?;
    }
    let tempo = cursor.number()?;
    skip_transitions(cursor, version, values, tempo)?;
    skip_mix_table_end(cursor, version)?;

    Ok(stated_tempo(tempo.into()))
}

fn skip_transitions(
    cursor: &mut Cursor<'_>,
    version: Version,
    values: &[u8],
    tempo: i32,
) -> Result<(), Unreadable> {
    let mut transitions = 0;
    for &value in values {
        transitions += usize::from(value.cast_signed() >= 0);
    }
    cursor.skip(transitions)?;

    if tempo >= 0 {
        let hide_tempo = usize::from(version.is_5_after_5_00());
        cursor.skip(1 + hide_tempo)?;
    }

    Ok(())
}

fn skip_mix_table_end(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let applies_to_all_tracks = 1;
    let wah = 1;

    if version.major >= 4 {
        cursor.skip(applies_to_all_tracks)?;
    }

    if version.is_5() {
        cursor.skip(wah)?;
    }

    if version.is_5_after_5_00() {
        let effect_and_category = 2;
        skip_texts(cursor, effect_and_category)?;
    }

    Ok(())
}

fn skip_notes(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let strings_played = cursor.byte()? & SEVEN_STRINGS;
    for _ in 0..strings_played.count_ones() {
        skip_note(cursor, version)?;
    }

    Ok(())
}

fn skip_note(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let flags = Flags(cursor.byte()?);
    let type_and_fret = 2 * usize::from(flags.has(NOTE_HAS_TYPE_AND_FRET));
    let velocity = usize::from(flags.has(NOTE_HAS_VELOCITY));
    let fingering = 2 * usize::from(flags.has(NOTE_HAS_FINGERING));
    let duration = note_duration(version, flags);
    let gp5_flags = usize::from(version.is_5());
    cursor.skip(type_and_fret + velocity + fingering + duration + gp5_flags)?;

    if flags.has(NOTE_HAS_EFFECTS) {
        skip_note_effects(cursor, version)?;
    }

    Ok(())
}

const fn note_duration(version: Version, flags: Flags) -> usize {
    let duration_and_tuplet = 2;
    let gp5_duration_percent = 8;

    if !flags.has(NOTE_HAS_DURATION_OR_PERCENT) {
        return 0;
    }

    if version.is_5() {
        return gp5_duration_percent;
    }

    duration_and_tuplet
}

fn skip_note_effects(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let gp3_grace = 4;

    if version.major == 3 {
        let flags = Flags(cursor.byte()?);

        if flags.has(NOTE_HAS_BEND) {
            skip_bend(cursor)?;
        }

        return cursor.skip(gp3_grace * usize::from(flags.has(NOTE_HAS_GRACE)));
    }

    skip_gp4_and_5_note_effects(cursor, version)
}

fn skip_gp4_and_5_note_effects(
    cursor: &mut Cursor<'_>,
    version: Version,
) -> Result<(), Unreadable> {
    let first = Flags(cursor.byte()?);
    let second = Flags(cursor.byte()?);
    let grace_size = if version.is_5() { 5 } else { 4 };
    let grace = grace_size * usize::from(first.has(NOTE_HAS_GRACE));
    let tremolo_picking = usize::from(second.has(NOTE_HAS_TREMOLO_PICKING));
    let slide = usize::from(second.has(NOTE_HAS_SLIDE));
    let trill = 2;

    if first.has(NOTE_HAS_BEND) {
        skip_bend(cursor)?;
    }
    cursor.skip(grace + tremolo_picking + slide)?;

    if second.has(NOTE_HAS_HARMONIC) {
        skip_harmonic(cursor, version)?;
    }

    cursor.skip(trill * usize::from(second.has(NOTE_HAS_TRILL)))
}

fn skip_harmonic(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    let kind = cursor.byte()?;
    let semitone_accidental_and_octave = 3;
    let fret = 1;

    if !version.is_5() {
        return Ok(());
    }

    match kind {
        GP5_ARTIFICIAL_HARMONIC => cursor.skip(semitone_accidental_and_octave),
        GP5_TAPPED_HARMONIC => cursor.skip(fret),
        _ => Ok(()),
    }
}

fn skip_bend(cursor: &mut Cursor<'_>) -> Result<(), Unreadable> {
    let kind_and_value = 1 + NUMBER;
    cursor.skip(kind_and_value)?;
    let points = cursor.count_up_to(usize::MAX)?;

    cursor.skip(points.saturating_mul(BEND_POINT))
}

fn skip_gp5_beat_end(cursor: &mut Cursor<'_>, version: Version) -> Result<(), Unreadable> {
    if !version.is_5() {
        return Ok(());
    }
    cursor.skip(1)?;
    let breaks_secondary_beams = Flags(cursor.byte()?).has(GP5_BEAT_BREAKS_SECONDARY_BEAMS);

    cursor.skip(usize::from(breaks_secondary_beams))
}

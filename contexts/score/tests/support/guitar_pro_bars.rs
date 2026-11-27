use crate::support::{Bytes, GuitarProFile};

const NO_CHANGE: i8 = -1;
const NO_TEMPO_CHANGE: i32 = -1;
const ALL_SEVEN_STRINGS: u8 = 0x7F;
const FIRST_STRING: u8 = 0x40;

impl GuitarProFile {
    pub(super) fn write_bars(&self, file: &mut Bytes) {
        if self.major == 5 {
            file.zeros(if self.minor == 0 { 2 } else { 1 });
        }
        let bar_tracks = self.bars * self.tracks.len();
        for bar_track in 0..bar_tracks {
            self.write_voice(file, bar_track / self.tracks.len());

            if self.major == 5 {
                file.count(0);
            }
            let line_break_omitted_at_the_end = bar_track + 1 == bar_tracks;

            if self.major == 5 && !line_break_omitted_at_the_end {
                file.zeros(1);
            }
        }
    }

    fn write_voice(&self, file: &mut Bytes, bar: usize) {
        let mut beats = Vec::new();

        if self.every_kind_of_beat {
            beats.extend(self.every_kind_of_beat());
        } else {
            beats.push(self.plain_beat());
        }
        for change in &self.tempo_changes {
            if change.bar == bar {
                beats.push(self.beat_with_mix_table(NO_CHANGE, change.beats_per_minute));
            }
        }
        file.count(beats.len());
        for beat in beats {
            file.0.extend(beat.0);
        }
    }

    fn plain_beat(&self) -> Bytes {
        let mut beat = Bytes::default();
        beat.0.extend([0, 0, FIRST_STRING, 0x20, 1, 3]);

        if self.major == 5 {
            beat.zeros(1);
        }
        self.end_beat(&mut beat, 0);

        beat
    }

    fn every_kind_of_beat(&self) -> Vec<Bytes> {
        vec![
            self.beat_with_everything(),
            self.beat_with_old_chord_and_no_effects(5),
            self.beat_with_new_chord(),
            self.beat_with_mix_table(5, NO_TEMPO_CHANGE),
        ]
    }

    fn beat_with_everything(&self) -> Bytes {
        let mut beat = Bytes::default();
        let status_dotted_tuplet_chord_text_effects = 0x40 | 0x01 | 0x20 | 0x02 | 0x04 | 0x08;
        beat.0
            .extend([status_dotted_tuplet_chord_text_effects, 0, 0]);
        beat.number(3);
        old_chord(&mut beat, 0);
        beat.text("Intro");
        self.beat_effects(&mut beat);
        beat.0.push(ALL_SEVEN_STRINGS);
        for string in 0..7 {
            self.note_with_everything(&mut beat, string);
        }
        self.end_beat(&mut beat, 0x08);

        beat
    }

    fn beat_with_old_chord_and_no_effects(&self, first_fret: i32) -> Bytes {
        let mut beat = Bytes::default();
        let effect_flags = if self.major == 3 { 1 } else { 2 };
        beat.0.extend([0x02 | 0x08, 0]);
        old_chord(&mut beat, first_fret);
        beat.zeros(effect_flags);
        beat.0.extend([FIRST_STRING, 0x08]);

        if self.major == 5 {
            beat.zeros(1);
        }
        beat.zeros(effect_flags);
        self.end_beat(&mut beat, 0);

        beat
    }

    fn beat_with_new_chord(&self) -> Bytes {
        let mut beat = Bytes::default();
        let new_chord = if self.major == 3 { 124 } else { 106 };
        beat.0.extend([0x02, 0, 1]);
        beat.zeros(new_chord);
        beat.0.push(0);
        self.end_beat(&mut beat, 0);

        beat
    }

    fn beat_effects(&self, beat: &mut Bytes) {
        if self.major == 3 {
            beat.0.extend([0x20 | 0x40, 1]);
            beat.number(0);
            beat.0.extend([0, 0]);

            return;
        }
        beat.0.extend([0x20 | 0x40, 0x04 | 0x02, 1]);
        bend(beat);
        beat.0.extend([0, 0, 0]);
    }

    fn beat_with_mix_table(&self, volume: i8, tempo: i32) -> Bytes {
        let mut beat = Bytes::default();
        beat.0.extend([0x10, 0]);
        self.mix_table(&mut beat, volume, tempo);
        beat.0.push(0);
        self.end_beat(&mut beat, 0);

        beat
    }

    fn mix_table(&self, beat: &mut Bytes, volume: i8, tempo: i32) {
        beat.zeros(if self.major == 5 { 1 + 16 } else { 1 });
        let values = [
            volume, NO_CHANGE, NO_CHANGE, NO_CHANGE, NO_CHANGE, NO_CHANGE,
        ];
        beat.0.extend(values.map(i8::cast_unsigned));

        if self.major == 5 {
            beat.text("");
        }
        beat.number(tempo);
        beat.zeros(values.iter().filter(|&&value| value >= 0).count());
        self.mix_table_end(beat, tempo);
    }

    fn mix_table_end(&self, beat: &mut Bytes, tempo: i32) {
        if tempo >= 0 {
            beat.zeros(if self.major == 5 && self.minor > 0 {
                2
            } else {
                1
            });
        }

        if self.major >= 4 {
            beat.zeros(1);
        }

        if self.major == 5 {
            beat.zeros(1);
        }

        if self.major == 5 && self.minor > 0 {
            beat.texts(&["", ""]);
        }
    }

    fn note_with_everything(&self, beat: &mut Bytes, string: usize) {
        let type_duration_velocity_fingering_effects = 0x20 | 0x01 | 0x10 | 0x80 | 0x08;
        beat.0.extend([type_duration_velocity_fingering_effects, 1]);

        if self.major < 5 {
            beat.zeros(2);
        }
        beat.0.extend([100, 3, 0, 0]);

        if self.major == 5 {
            beat.zeros(8 + 1);
        }
        self.note_effects(beat, string);
    }

    fn note_effects(&self, beat: &mut Bytes, string: usize) {
        if self.major == 3 {
            beat.0.push(0x01 | 0x10);
            bend(beat);
            beat.zeros(4);

            return;
        }
        beat.0.extend([0x01 | 0x10, 0x04 | 0x08 | 0x10 | 0x20]);
        bend(beat);
        beat.zeros(if self.major == 5 { 5 } else { 4 });
        beat.zeros(1 + 1);
        self.harmonic(beat, string);
        beat.zeros(2);
    }

    fn harmonic(&self, beat: &mut Bytes, string: usize) {
        let natural_artificial_or_tapped = [1, 2, 3][string % 3];
        beat.0.push(natural_artificial_or_tapped);

        if self.major < 5 {
            return;
        }
        match natural_artificial_or_tapped {
            2 => beat.zeros(3),
            3 => beat.zeros(1),
            _ => {}
        }
    }

    fn end_beat(&self, beat: &mut Bytes, beam_flags: u8) {
        if self.major < 5 {
            return;
        }
        beat.0.extend([0, beam_flags]);

        if beam_flags & 0x08 != 0 {
            beat.zeros(1);
        }
    }
}

fn old_chord(beat: &mut Bytes, first_fret: i32) {
    beat.0.push(0);
    beat.text("Am");
    beat.number(first_fret);

    if first_fret != 0 {
        beat.zeros(6 * 4);
    }
}

fn bend(beat: &mut Bytes) {
    beat.zeros(1 + 4);
    beat.number(2);
    beat.zeros(2 * 9);
}

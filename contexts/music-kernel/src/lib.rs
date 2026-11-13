use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pitch(u8);

impl Pitch {
    pub const fn midi(number: u8) -> Self {
        Self(number)
    }

    pub const fn midi_number(self) -> u8 {
        self.0
    }

    const fn semitones_above(self, lower: Self) -> u8 {
        self.0.wrapping_sub(lower.0)
    }

    const fn note_name(self) -> &'static str {
        const NOTE_NAMES: [&str; 12] = [
            "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
        ];

        NOTE_NAMES[(self.0 % 12) as usize]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tuning {
    pub lowest_string_first: Vec<Pitch>,
}

impl Tuning {
    pub const fn of(lowest_string_first: Vec<Pitch>) -> Self {
        Self {
            lowest_string_first,
        }
    }

    pub fn of_midi_numbers(lowest_string_first: &[u8]) -> Self {
        let mut pitches = Vec::new();
        for &midi_number in lowest_string_first {
            pitches.push(Pitch::midi(midi_number));
        }

        Self::of(pitches)
    }

    pub const fn strings(&self) -> usize {
        self.lowest_string_first.len()
    }

    pub fn name(&self) -> TuningName {
        let Some(&lowest) = self.lowest_string_first.first() else {
            return TuningName::Custom;
        };
        let Some(standard) = standard_intervals(self.strings()) else {
            return TuningName::Custom;
        };
        let intervals = self.intervals_above(lowest);

        if intervals == standard {
            return TuningName::Standard(lowest);
        }

        if is_dropped(&intervals, standard) {
            return TuningName::Drop(lowest);
        }

        TuningName::Custom
    }

    fn intervals_above(&self, lowest: Pitch) -> Vec<u8> {
        let mut intervals = Vec::new();
        for pitch in &self.lowest_string_first {
            intervals.push(pitch.semitones_above(lowest));
        }

        intervals
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuningName {
    Standard(Pitch),
    Drop(Pitch),
    Custom,
}

impl fmt::Display for TuningName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard(lowest) => write!(formatter, "{} Standard", lowest.note_name()),
            Self::Drop(lowest) => write!(formatter, "Drop {}", lowest.note_name()),
            Self::Custom => write!(formatter, "Custom"),
        }
    }
}

const fn standard_intervals(strings: usize) -> Option<&'static [u8]> {
    match strings {
        4 => Some(&[0, 5, 10, 15]),
        5 => Some(&[0, 5, 10, 15, 20]),
        6 => Some(&[0, 5, 10, 15, 19, 24]),
        7 => Some(&[0, 5, 10, 15, 20, 24, 29]),
        _ => None,
    }
}

fn is_dropped(intervals: &[u8], standard: &[u8]) -> bool {
    let Some((lowest, higher)) = intervals.split_first() else {
        return false;
    };
    let mut raised_back = vec![*lowest];
    for interval in higher {
        raised_back.push(interval.wrapping_sub(2));
    }

    raised_back == standard
}

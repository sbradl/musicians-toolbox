use std::io::BufRead;

use mt_music_kernel::Tuning;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::{MOST_TRACKS, ScoreHeader, ScoreXml, Track, Unreadable, pitch_of};

const MOST_DEPTH: usize = 64;
const MOST_ELEMENTS: usize = 1_000_000;
const MOST_STRINGS: usize = 16;
const TRACK: [&str; 3] = ["GPIF", "Tracks", "Track"];
const TRACKS: [&str; 2] = ["GPIF", "Tracks"];
const TUNING_PITCHES: [&str; 2] = ["Property name=Tuning", "Pitches"];
const INSTRUMENT_TYPE: [&str; 2] = ["InstrumentSet", "Type"];
const DRUM_KIT: &str = "drumKit";

#[derive(Debug, Default)]
struct HeaderSoFar {
    open_elements: Vec<String>,
    elements_seen: usize,
    tracks: Vec<TrackSoFar>,
}

#[derive(Debug, PartialEq, Eq)]
enum Reading {
    GoesOn,
    TracksComplete,
}

#[derive(Debug, Default)]
struct TrackSoFar {
    tuning: Option<Tuning>,
    is_drums: bool,
}

impl<Xml: BufRead> ScoreXml<Xml> {
    pub(super) fn read_header(self) -> Result<ScoreHeader, Unreadable> {
        let mut reader = Reader::from_reader(self.0);
        let mut header = HeaderSoFar::default();
        let mut element = Vec::new();
        loop {
            element.clear();
            let Ok(event) = reader.read_event_into(&mut element) else {
                return Err(Unreadable);
            };

            if header.read(event)? == Reading::TracksComplete {
                return Ok(header.into_score_header());
            }
        }
    }
}

impl HeaderSoFar {
    fn read(&mut self, event: Event<'_>) -> Result<Reading, Unreadable> {
        match event {
            Event::Start(element) => self.open(&element)?,
            Event::Empty(element) => {
                self.open(&element)?;

                return Ok(self.close());
            }
            Event::Text(text) => self.read_text(&text)?,
            Event::End(_) => return Ok(self.close()),
            Event::Eof => return Err(Unreadable),
            _ => {}
        }

        Ok(Reading::GoesOn)
    }

    fn open(&mut self, element: &BytesStart<'_>) -> Result<(), Unreadable> {
        self.elements_seen += 1;
        self.open_elements.push(tag_of(element));

        if self.open_elements.len() > MOST_DEPTH || self.elements_seen > MOST_ELEMENTS {
            return Err(Unreadable);
        }

        if self.open_elements == TRACK {
            self.tracks.push(TrackSoFar::default());
        }

        if self.tracks.len() > MOST_TRACKS {
            return Err(Unreadable);
        }

        Ok(())
    }

    fn close(&mut self) -> Reading {
        let closes_tracks = self.open_elements == TRACKS;
        drop(self.open_elements.pop());

        if closes_tracks {
            return Reading::TracksComplete;
        }

        Reading::GoesOn
    }

    fn read_text(&mut self, text: &str) -> Result<(), Unreadable> {
        let names_instrument = self.ends_with(&INSTRUMENT_TYPE);
        let lists_tuning = self.ends_with(&TUNING_PITCHES);
        let Some(track) = self.track_being_read() else {
            return Ok(());
        };

        if names_instrument && text == DRUM_KIT {
            track.is_drums = true;
        }

        if lists_tuning && track.tuning.is_none() {
            track.tuning = Some(tuning_of(text)?);
        }

        Ok(())
    }

    fn track_being_read(&mut self) -> Option<&mut TrackSoFar> {
        if self.is_inside(&TRACK) {
            return self.tracks.last_mut();
        }

        None
    }

    fn is_inside(&self, path: &[&str]) -> bool {
        let open = &self.open_elements;

        open.len() > path.len() && open[..path.len()] == *path
    }

    fn ends_with(&self, path: &[&str]) -> bool {
        let open = &self.open_elements;

        open.len() >= path.len() && open[open.len() - path.len()..] == *path
    }

    fn into_score_header(self) -> ScoreHeader {
        ScoreHeader {
            title: None,
            artist: None,
            album: None,
            tracks: self
                .tracks
                .into_iter()
                .map(TrackSoFar::into_track)
                .collect(),
        }
    }
}

impl TrackSoFar {
    fn into_track(self) -> Track {
        if self.is_drums {
            return Track { tuning: None };
        }

        Track {
            tuning: self.tuning,
        }
    }
}

fn tag_of(element: &BytesStart<'_>) -> String {
    let name = element.name().as_ref().to_owned();

    if name != "Property" {
        return name;
    }
    let Ok(Some(property)) = element.try_get_attribute("name") else {
        return name;
    };

    format!("Property name={}", property.value)
}

fn tuning_of(pitches: &str) -> Result<Tuning, Unreadable> {
    let mut lowest_string_first = Vec::new();
    for midi_number in pitches.split_whitespace() {
        let Ok(midi_number) = midi_number.parse() else {
            return Err(Unreadable);
        };
        lowest_string_first.push(pitch_of(midi_number)?);
    }

    if lowest_string_first.is_empty() || lowest_string_first.len() > MOST_STRINGS {
        return Err(Unreadable);
    }

    Ok(Tuning::of(lowest_string_first))
}

use std::io::BufRead;

use mt_music_kernel::Tuning;
use quick_xml::Reader;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesRef, Event};

use crate::{
    MOST_STRINGS, MOST_TRACKS, OpenElements, ScoreHeader, Track, TuxGuitarXml, Unreadable,
    pitch_of, stated,
};

const SONG: [&str; 2] = ["TuxGuitarFile", "TGSong"];
const SONG_NAMES: [[&str; 3]; 3] = [
    ["TuxGuitarFile", "TGSong", "name"],
    ["TuxGuitarFile", "TGSong", "artist"],
    ["TuxGuitarFile", "TGSong", "album"],
];
const CHANNEL: [&str; 3] = ["TuxGuitarFile", "TGSong", "TGChannel"];
const CHANNEL_ID: [&str; 4] = ["TuxGuitarFile", "TGSong", "TGChannel", "id"];
const CHANNEL_BANK: [&str; 4] = ["TuxGuitarFile", "TGSong", "TGChannel", "bank"];
const TRACK: [&str; 3] = ["TuxGuitarFile", "TGSong", "TGTrack"];
const TRACK_CHANNEL_ID: [&str; 4] = ["TuxGuitarFile", "TGSong", "TGTrack", "channelId"];
const TRACK_STRING: [&str; 4] = ["TuxGuitarFile", "TGSong", "TGTrack", "TGString"];
const PERCUSSION_BANK: &str = "128";

#[derive(Debug, Default)]
struct SongSoFar {
    open_elements: OpenElements,
    names: [String; 3],
    channels: Vec<ChannelSoFar>,
    tracks: Vec<TrackSoFar>,
}

#[derive(Debug, Default)]
struct ChannelSoFar {
    id: String,
    bank: String,
}

#[derive(Debug, Default)]
struct TrackSoFar {
    channel_id: String,
    highest_string_first: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum Reading {
    GoesOn,
    SongComplete,
}

impl<Xml: BufRead> TuxGuitarXml<Xml> {
    pub(super) fn read_header(self) -> Result<ScoreHeader, Unreadable> {
        let mut reader = Reader::from_reader(self.0);
        let mut song = SongSoFar::default();
        let mut element = Vec::new();
        loop {
            element.clear();
            let Ok(event) = reader.read_event_into(&mut element) else {
                return Err(Unreadable);
            };

            if song.read(event)? == Reading::SongComplete {
                return song.into_score_header();
            }
        }
    }
}

impl SongSoFar {
    fn read(&mut self, event: Event<'_>) -> Result<Reading, Unreadable> {
        match event {
            Event::Start(element) => self.open(element.name().as_ref())?,
            Event::Empty(element) => {
                self.open(element.name().as_ref())?;

                return Ok(self.close());
            }
            Event::Text(text) => self.read_text(&text),
            Event::CData(text) => self.read_text(&text),
            Event::GeneralRef(reference) => self.read_text(&resolved(&reference)?),
            Event::End(_) => return Ok(self.close()),
            Event::Eof => return Err(Unreadable),
            _ => {}
        }

        Ok(Reading::GoesOn)
    }

    fn open(&mut self, name: &str) -> Result<(), Unreadable> {
        self.open_elements.open(name.to_owned())?;
        self.start_channel_track_or_string();

        if self.tracks.len() > MOST_TRACKS {
            return Err(Unreadable);
        }

        Ok(())
    }

    fn start_channel_track_or_string(&mut self) {
        if self.open_elements.are(&CHANNEL) {
            self.channels.push(ChannelSoFar::default());
        }

        if self.open_elements.are(&TRACK) {
            self.tracks.push(TrackSoFar::default());
        }

        if self.open_elements.are(&TRACK_STRING)
            && let Some(track) = self.tracks.last_mut()
        {
            track.highest_string_first.push(String::new());
        }
    }

    fn close(&mut self) -> Reading {
        let closes_song = self.open_elements.are(&SONG);
        self.open_elements.close();

        if closes_song {
            return Reading::SongComplete;
        }

        Reading::GoesOn
    }

    fn read_text(&mut self, text: &str) {
        if let Some(read_so_far) = self.text_being_read() {
            read_so_far.push_str(text);
        }
    }

    fn text_being_read(&mut self) -> Option<&mut String> {
        if let Some(name) = self.song_name_being_read() {
            return Some(&mut self.names[name]);
        }
        let open = &self.open_elements;

        if open.are(&CHANNEL_ID) || open.are(&CHANNEL_BANK) {
            let reads_id = open.are(&CHANNEL_ID);
            let channel = self.channels.last_mut()?;

            return Some(if reads_id {
                &mut channel.id
            } else {
                &mut channel.bank
            });
        }

        self.track_text_being_read()
    }

    fn song_name_being_read(&self) -> Option<usize> {
        for (name, path) in SONG_NAMES.iter().enumerate() {
            if self.open_elements.are(path) {
                return Some(name);
            }
        }

        None
    }

    fn track_text_being_read(&mut self) -> Option<&mut String> {
        let reads_channel_id = self.open_elements.are(&TRACK_CHANNEL_ID);
        let reads_string = self.open_elements.are(&TRACK_STRING);
        let track = self.tracks.last_mut()?;

        if reads_channel_id {
            return Some(&mut track.channel_id);
        }

        if reads_string {
            return track.highest_string_first.last_mut();
        }

        None
    }

    fn into_score_header(self) -> Result<ScoreHeader, Unreadable> {
        let mut tracks = Vec::new();
        for track in &self.tracks {
            tracks.push(self.track_of(track)?);
        }
        let [title, artist, album] = self.names;

        Ok(ScoreHeader {
            title: stated(&title),
            artist: stated(&artist),
            album: stated(&album),
            tracks,
            written_tempos: Vec::new(),
        })
    }

    fn track_of(&self, track: &TrackSoFar) -> Result<Track, Unreadable> {
        if self.is_drum_channel(&track.channel_id) {
            return Ok(Track { tuning: None });
        }

        Ok(Track {
            tuning: tuning_of(&track.highest_string_first)?,
        })
    }

    fn is_drum_channel(&self, id: &str) -> bool {
        for channel in &self.channels {
            if channel.id.trim() == id.trim() {
                return channel.bank.trim() == PERCUSSION_BANK;
            }
        }

        false
    }
}

fn resolved(reference: &BytesRef<'_>) -> Result<String, Unreadable> {
    if let Ok(Some(character)) = reference.resolve_char_ref() {
        return Ok(character.to_string());
    }
    let Some(entity) = resolve_predefined_entity(reference) else {
        return Err(Unreadable);
    };

    Ok(entity.to_owned())
}

fn tuning_of(highest_string_first: &[String]) -> Result<Option<Tuning>, Unreadable> {
    if highest_string_first.len() > MOST_STRINGS {
        return Err(Unreadable);
    }
    let mut lowest_string_first = Vec::new();
    for midi_number in highest_string_first.iter().rev() {
        if midi_number.trim().is_empty() {
            continue;
        }
        let Ok(midi_number) = midi_number.trim().parse() else {
            return Err(Unreadable);
        };
        lowest_string_first.push(pitch_of(midi_number)?);
    }

    if lowest_string_first.is_empty() {
        return Ok(None);
    }

    Ok(Some(Tuning::of(lowest_string_first)))
}

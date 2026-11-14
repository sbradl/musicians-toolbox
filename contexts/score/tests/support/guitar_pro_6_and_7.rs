use crate::support::zip_holding;

const SECTOR: usize = 0x1000;
const SECTOR_PAST_THE_END: u32 = 0x7FFF_FFFF;
const FILE_ENTRY: u32 = 2;
const FILE_NAME_FIELD: usize = 0x04;
const FILE_SIZE_FIELD: usize = 0x8C;
const SECTOR_LIST_FIELD: usize = 0x94;
const LONGEST_COPY: usize = (1 << 12) - 1;

#[derive(Debug)]
pub struct ScoreXml {
    guitar_pro: u8,
    before_tracks: String,
    tracks: Vec<String>,
    after_tracks: String,
}

impl ScoreXml {
    #[must_use]
    pub const fn guitar_pro(major: u8) -> Self {
        Self {
            guitar_pro: major,
            before_tracks: String::new(),
            tracks: Vec::new(),
            after_tracks: String::new(),
        }
    }

    #[must_use]
    pub fn with_strings(mut self, lowest_string_first: &[u8]) -> Self {
        let instrument = "<InstrumentSet><Type>electricGuitar</Type></InstrumentSet>";
        self.add_track(instrument, &self.properties(lowest_string_first));

        self
    }

    #[must_use]
    pub fn with_pitches_written(mut self, pitches: &str) -> Self {
        let instrument = "<InstrumentSet><Type>electricGuitar</Type></InstrumentSet>";
        let properties = self.properties_listing(&tuning_property_listing(pitches));
        self.add_track(instrument, &properties);

        self
    }

    #[must_use]
    pub fn with_drums(mut self) -> Self {
        let instrument = "<InstrumentSet><Type>drumKit</Type></InstrumentSet>";
        let properties = if self.guitar_pro == 6 {
            String::new()
        } else {
            self.properties(&[0; 6])
        };
        self.add_track(instrument, &properties);

        self
    }

    fn properties(&self, lowest_string_first: &[u8]) -> String {
        self.properties_listing(&tuning_property(lowest_string_first))
    }

    fn properties_listing(&self, tuning: &str) -> String {
        if self.guitar_pro == 6 {
            return format!("<Properties>{tuning}</Properties>");
        }

        format!("<Staves><Staff><Properties>{tuning}</Properties></Staff></Staves>")
    }

    #[must_use]
    pub fn preceded_by(mut self, xml_before_tracks: &str) -> Self {
        xml_before_tracks.clone_into(&mut self.before_tracks);

        self
    }

    #[must_use]
    pub fn followed_by(mut self, xml_after_tracks: &str) -> Self {
        xml_after_tracks.clone_into(&mut self.after_tracks);

        self
    }

    fn add_track(&mut self, instrument: &str, properties: &str) {
        let id = self.tracks.len();
        self.tracks.push(format!(
            "<Track id=\"{id}\"><Name><![CDATA[Invented <Track> {id}]]></Name>{instrument}{properties}</Track>"
        ));
    }

    #[must_use]
    pub fn xml(&self) -> Vec<u8> {
        let track_ids = (0..self.tracks.len())
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GPIF>{}\
             <GPVersion>{}</GPVersion>\
             <Score><Title><![CDATA[Invented Song]]></Title></Score>\
             <MasterTrack><Tracks>{track_ids}</Tracks></MasterTrack>\
             <Tracks>{}</Tracks>{}\
             <MasterBars><MasterBar><Time>4/4</Time></MasterBar></MasterBars></GPIF>",
            self.before_tracks,
            self.guitar_pro,
            self.tracks.concat(),
            self.after_tracks,
        )
        .into_bytes()
    }
}

fn tuning_property(lowest_string_first: &[u8]) -> String {
    let pitches = lowest_string_first
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(" ");

    tuning_property_listing(&pitches)
}

fn tuning_property_listing(pitches: &str) -> String {
    format!(
        "<Property name=\"CapoFret\"><Fret>0</Fret></Property>\
         <Property name=\"Tuning\"><Pitches>{pitches}</Pitches></Property>"
    )
}

#[must_use]
pub fn guitar_pro_7_file(score_xml: &[u8]) -> Vec<u8> {
    zip_holding(&[
        ("VERSION", b"7.0"),
        ("Content/Assets/backing.mp3", b"invented audio"),
        ("Content/score.gpif", score_xml),
    ])
}

#[must_use]
pub fn guitar_pro_6_file(score_xml: &[u8]) -> Vec<u8> {
    compressed_guitar_pro_6_file(&file_system_holding(score_xml))
}

#[must_use]
pub fn uncompressed_guitar_pro_6_file(score_xml: &[u8]) -> Vec<u8> {
    [b"BCFS".as_slice(), &file_system_holding(score_xml)].concat()
}

#[must_use]
pub fn guitar_pro_6_file_stating_length(score_xml: &[u8], length: usize) -> Vec<u8> {
    let uncompressed = uncompressed_guitar_pro_6_file(score_xml);

    guitar_pro_6_file_of_compressed_bytes(length, &compressed(&uncompressed))
}

#[must_use]
pub fn compressed_guitar_pro_6_file(file_system: &[u8]) -> Vec<u8> {
    guitar_pro_6_file_packing(&[b"BCFS".as_slice(), file_system].concat())
}

#[must_use]
pub fn guitar_pro_6_file_packing(uncompressed: &[u8]) -> Vec<u8> {
    guitar_pro_6_file_of_compressed_bytes(uncompressed.len(), &compressed(uncompressed))
}

#[must_use]
pub fn guitar_pro_6_file_of_compressed_bytes(stated_length: usize, bytes: &[u8]) -> Vec<u8> {
    let length = u32::try_from(stated_length).unwrap_or(u32::MAX);

    [b"BCFZ".as_slice(), &length.to_le_bytes(), bytes].concat()
}

fn file_system_holding(score_xml: &[u8]) -> Vec<u8> {
    FileSystem::default()
        .with_file("misc.xml", b"<Misc/>")
        .with_file("score.gpif", score_xml)
        .bytes()
}

#[derive(Debug, Default)]
pub struct FileSystem {
    files: Vec<FileInSystem>,
}

#[derive(Debug)]
struct FileInSystem {
    name: String,
    stated_size: usize,
    contents: Vec<u8>,
    is_past_the_end: bool,
}

impl FileSystem {
    #[must_use]
    pub fn with_file(self, name: &str, contents: &[u8]) -> Self {
        self.with_file_stating_size(name, contents.len(), contents)
    }

    #[must_use]
    pub fn with_file_stating_size(mut self, name: &str, size: usize, contents: &[u8]) -> Self {
        self.files.push(FileInSystem {
            name: name.to_owned(),
            stated_size: size,
            contents: contents.to_vec(),
            is_past_the_end: false,
        });

        self
    }

    #[must_use]
    pub fn with_file_past_the_end(mut self, name: &str, size: usize) -> Self {
        self.files.push(FileInSystem {
            name: name.to_owned(),
            stated_size: size,
            contents: Vec::new(),
            is_past_the_end: true,
        });

        self
    }

    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        let mut sectors = vec![vec![0; SECTOR]];
        for file in &self.files {
            let entry_sector = sectors.len();
            sectors.push(Vec::new());
            let mut sector_list = Vec::new();
            for chunk in file.contents.chunks(SECTOR) {
                sector_list.push(u32::try_from(sectors.len()).unwrap_or(u32::MAX));
                let mut sector = chunk.to_vec();
                sector.resize(SECTOR, 0);
                sectors.push(sector);
            }

            if file.is_past_the_end {
                sector_list.push(SECTOR_PAST_THE_END);
            }
            sectors[entry_sector] = file_entry(file, &sector_list);
        }

        sectors.concat()
    }
}

fn file_entry(file: &FileInSystem, sector_list: &[u32]) -> Vec<u8> {
    let size = u32::try_from(file.stated_size).unwrap_or(u32::MAX);
    let mut entry = vec![0; SECTOR];
    entry[..4].copy_from_slice(&FILE_ENTRY.to_le_bytes());
    let name = file.name.as_bytes();
    entry[FILE_NAME_FIELD..FILE_NAME_FIELD + name.len()].copy_from_slice(name);
    entry[FILE_SIZE_FIELD..FILE_SIZE_FIELD + 4].copy_from_slice(&size.to_le_bytes());
    for (index, sector) in sector_list.iter().enumerate() {
        let at = SECTOR_LIST_FIELD + 4 * index;
        entry[at..at + 4].copy_from_slice(&sector.to_le_bytes());
    }

    entry
}

fn compressed(uncompressed: &[u8]) -> Vec<u8> {
    let mut bits = Bits::default();
    let mut position = 0;
    while position < uncompressed.len() {
        let zeros = repeated_zeros(uncompressed, position);

        if zeros >= 4 {
            bits.copy(zeros);
            position += zeros;
        } else {
            let literals = (uncompressed.len() - position).min(3);
            bits.literals(&uncompressed[position..position + literals]);
            position += literals;
        }
    }

    bits.bytes
}

fn repeated_zeros(uncompressed: &[u8], position: usize) -> usize {
    let is_zero = |byte: &&u8| **byte == 0;
    let ahead = uncompressed[position..].iter().take_while(is_zero).count();
    let behind = uncompressed[..position]
        .iter()
        .rev()
        .take_while(is_zero)
        .count();

    ahead.min(behind).min(LONGEST_COPY)
}

#[derive(Debug, Default)]
struct Bits {
    bytes: Vec<u8>,
    used_in_last_byte: u8,
}

impl Bits {
    fn copy(&mut self, length: usize) {
        let word_size = 12;
        self.push(1, 1);
        self.push(word_size, 4);
        self.push_reversed(length, word_size);
        self.push_reversed(length, word_size);
    }

    fn literals(&mut self, literals: &[u8]) {
        self.push(0, 1);
        self.push_reversed(literals.len(), 2);
        for &byte in literals {
            self.push(usize::from(byte), 8);
        }
    }

    fn push(&mut self, value: usize, count: usize) {
        for index in (0..count).rev() {
            self.bit(value >> index & 1 == 1);
        }
    }

    fn push_reversed(&mut self, value: usize, count: usize) {
        for index in 0..count {
            self.bit(value >> index & 1 == 1);
        }
    }

    fn bit(&mut self, set: bool) {
        if self.used_in_last_byte.is_multiple_of(8) {
            self.bytes.push(0);
            self.used_in_last_byte = 0;
        }

        if let Some(last) = self.bytes.last_mut() {
            *last |= u8::from(set) << (7 - self.used_in_last_byte);
        }
        self.used_in_last_byte += 1;
    }
}

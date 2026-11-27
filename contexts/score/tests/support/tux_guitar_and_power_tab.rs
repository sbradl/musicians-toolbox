#[derive(Debug, Default)]
pub struct TuxGuitar1File {
    title: String,
    artist: String,
    album: String,
}

impl TuxGuitar1File {
    #[must_use]
    pub fn titled(mut self, title: &str) -> Self {
        title.clone_into(&mut self.title);

        self
    }

    #[must_use]
    pub fn by(mut self, artist: &str) -> Self {
        artist.clone_into(&mut self.artist);

        self
    }

    #[must_use]
    pub fn on_album(mut self, album: &str) -> Self {
        album.clone_into(&mut self.album);

        self
    }

    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        let mut file = Vec::new();
        for text in [
            "TuxGuitar File Format - 1.5",
            &self.title,
            &self.artist,
            &self.album,
            "made up for the tests",
        ] {
            write_utf16_text(&mut file, text);
        }
        file.resize(file.len() + 64, 0);

        file
    }
}

fn write_utf16_text(file: &mut Vec<u8>, text: &str) {
    let units: Vec<u16> = text.encode_utf16().take(usize::from(u8::MAX)).collect();
    file.push(u8::try_from(units.len()).unwrap_or(u8::MAX));
    for unit in units {
        file.extend(unit.to_be_bytes());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    PublicAudio,
    PublicVideo,
    Unreleased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSize {
    OneByte,
    FourBytes,
    Unicode,
}

#[derive(Debug)]
pub struct PowerTabFile {
    version: u16,
    is_lesson: bool,
    title: String,
    artist: String,
    album: String,
    release: Release,
    name_size: NameSize,
}

impl PowerTabFile {
    #[must_use]
    pub const fn version(version: u16) -> Self {
        Self {
            version,
            is_lesson: false,
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            release: Release::PublicAudio,
            name_size: NameSize::OneByte,
        }
    }

    #[must_use]
    pub fn titled(mut self, title: &str) -> Self {
        title.clone_into(&mut self.title);

        self
    }

    #[must_use]
    pub fn by(mut self, artist: &str) -> Self {
        artist.clone_into(&mut self.artist);

        self
    }

    #[must_use]
    pub fn on_album(mut self, album: &str, release: Release) -> Self {
        album.clone_into(&mut self.album);
        self.release = release;

        self
    }

    #[must_use]
    pub const fn as_lesson(mut self) -> Self {
        self.is_lesson = true;

        self
    }

    #[must_use]
    pub const fn with_names_sized(mut self, name_size: NameSize) -> Self {
        self.name_size = name_size;

        self
    }

    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        let mut file = b"ptab".to_vec();
        file.extend(self.version.to_le_bytes());

        if self.is_lesson {
            file.push(1);
            self.write_name(&mut file, &self.title);
        } else {
            self.write_song_information(&mut file);
        }
        file.resize(file.len() + 64, 0);

        file
    }

    fn write_song_information(&self, file: &mut Vec<u8>) {
        let song = 0;
        let content_type = 0;
        file.extend([song, content_type]);
        self.write_name(file, &self.title);
        self.write_name(file, &self.artist);
        match self.release {
            Release::PublicAudio => {
                let album_type = 0;
                file.extend([0, album_type]);
                self.write_name(file, &self.album);
            }
            Release::PublicVideo => {
                file.push(1);
                self.write_name(file, &self.album);
            }
            Release::Unreleased => file.push(3),
        }
    }

    fn write_name(&self, file: &mut Vec<u8>, name: &str) {
        let length = u32::try_from(name.len()).unwrap_or(u32::MAX);
        match self.name_size {
            NameSize::OneByte if name.len() < 0xFF => file.push(u8::try_from(length).unwrap_or(0)),
            NameSize::OneByte => {
                file.push(0xFF);
                file.extend(u16::try_from(length).unwrap_or(0).to_le_bytes());
            }
            NameSize::FourBytes => {
                file.extend([0xFF, 0xFF, 0xFF]);
                file.extend(length.to_le_bytes());
            }
            NameSize::Unicode => file.extend([0xFF, 0xFE, 0xFF]),
        }
        file.extend(name.as_bytes());
    }
}

#[derive(Debug, Default)]
pub struct TuxGuitar2Xml {
    title: String,
    artist: String,
    channels: Vec<String>,
    tracks: Vec<String>,
}

const PERCUSSION_BANK: u8 = 128;

impl TuxGuitar2Xml {
    #[must_use]
    pub fn titled(mut self, title: &str) -> Self {
        title.clone_into(&mut self.title);

        self
    }

    #[must_use]
    pub fn by(mut self, artist: &str) -> Self {
        artist.clone_into(&mut self.artist);

        self
    }

    #[must_use]
    pub fn with_strings(self, lowest_string_first: &[u8]) -> Self {
        let mut strings = Vec::new();
        for pitch in lowest_string_first.iter().rev() {
            strings.push(pitch.to_string());
        }

        self.with_strings_written(&strings, 0)
    }

    #[must_use]
    pub fn with_drums(self) -> Self {
        let strings = ["0"; 6].map(str::to_owned);

        self.with_strings_written(&strings, PERCUSSION_BANK)
    }

    #[must_use]
    pub fn with_strings_written(mut self, highest_first: &[String], bank: u8) -> Self {
        let channel = self.channels.len() + 1;
        self.channels.push(format!(
            "<TGChannel><id>{channel}</id><bank>{bank}</bank><program>25</program>\
             <volume>127</volume><name>Channel {channel}</name></TGChannel>"
        ));
        let mut strings = String::new();
        for pitch in highest_first {
            strings.push_str("<TGString>");
            strings.push_str(pitch);
            strings.push_str("</TGString>");
        }
        self.tracks.push(format!(
            "<TGTrack maxFret=\"24\"><name>Invented Track</name><channelId>{channel}</channelId>\
             <color R=\"255\" G=\"0\" B=\"0\"/>{strings}<TGLyric from=\"1\"/>\
             <TGMeasure><TGBeat><preciseStart>960</preciseStart></TGBeat></TGMeasure></TGTrack>"
        ));

        self
    }

    #[must_use]
    pub fn xml(&self) -> Vec<u8> {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n\
             <TuxGuitarFile><TGVersion major=\"2\" minor=\"0\" revision=\"0\"/><TGSong>\
             <name>{}</name><artist>{}</artist><album>Saltglass</album><author/>\
             <comments>made up for the tests</comments>{}\
             <TGMeasureHeader><tempo>120</tempo></TGMeasureHeader>{}</TGSong></TuxGuitarFile>",
            self.title,
            self.artist,
            self.channels.concat(),
            self.tracks.concat(),
        )
        .into_bytes()
    }
}

#[must_use]
pub fn tux_guitar_2_file(content: &[u8]) -> Vec<u8> {
    crate::support::zip_holding(&[
        ("version.txt", b"TuxGuitar_file_format 2.0"),
        ("content.xml", content),
    ])
}

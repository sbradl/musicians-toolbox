use std::io::{self, BufReader, Read};

use crate::{GuitarPro6File, LARGEST_SCORE_XML, ScoreHeader, ScoreXml, Unreadable};

const COMPRESSED: &[u8] = b"BCFZ";
const UNCOMPRESSED: &[u8] = b"BCFS";
const CONTAINER_NAME_LENGTH: usize = 4;
const NUMBER: usize = 4;
const SECTOR: usize = 0x1000;
const READ_AHEAD: usize = SECTOR / 2;
const FILE_ENTRY_KIND: usize = 2;
const FILE_NAME_FIELD: usize = 0x04;
const LONGEST_FILE_NAME: usize = 127;
const FILE_SIZE_FIELD: usize = 0x8C;
const SECTOR_LIST_FIELD: usize = 0x94;
const SCORE_XML_NAME: &[u8] = b"score.gpif";

impl<'file> GuitarPro6File<'file> {
    pub(super) fn recognized(tab_file: &'file [u8]) -> Option<Self> {
        if tab_file.starts_with(COMPRESSED) || tab_file.starts_with(UNCOMPRESSED) {
            return Some(Self(tab_file));
        }

        None
    }

    pub(super) fn read_header(&self) -> Result<ScoreHeader, Unreadable> {
        let mut file_system = self.file_system()?;
        let score_xml = file_system.score_xml()?;

        ScoreXml(BufReader::with_capacity(READ_AHEAD, score_xml)).read_header()
    }

    fn file_system(&self) -> Result<FileSystem<'file>, Unreadable> {
        if self.0.len() > LARGEST_SCORE_XML {
            return Err(Unreadable);
        }

        if self.0.starts_with(UNCOMPRESSED) {
            return Ok(FileSystem {
                unpacked: self.0.to_vec(),
                length: self.0.len(),
                bits: Bits::over(&[]),
            });
        }
        let Some((length, compressed)) = self.0[CONTAINER_NAME_LENGTH..].split_first_chunk() else {
            return Err(Unreadable);
        };

        FileSystem::compressed(number_from(*length)?, compressed)
    }
}

#[derive(Debug)]
struct FileSystem<'file> {
    unpacked: Vec<u8>,
    length: usize,
    bits: Bits<'file>,
}

impl<'file> FileSystem<'file> {
    fn compressed(length: usize, compressed: &'file [u8]) -> Result<Self, Unreadable> {
        if length > LARGEST_SCORE_XML {
            return Err(Unreadable);
        }
        let mut file_system = Self {
            unpacked: Vec::new(),
            length,
            bits: Bits::over(compressed),
        };

        if file_system.unpacked_bytes(0, CONTAINER_NAME_LENGTH) != UNCOMPRESSED {
            return Err(Unreadable);
        }

        Ok(file_system)
    }

    fn score_xml(&mut self) -> Result<FileReader<'_, 'file>, Unreadable> {
        let mut entry = SECTOR;
        while !self.bytes(entry, NUMBER).is_empty() {
            if self.is_score_xml_entry(entry) {
                return self.file_at(entry);
            }
            entry += SECTOR;
        }

        Err(Unreadable)
    }

    fn is_score_xml_entry(&mut self, entry: usize) -> bool {
        let is_file = self.number_at(entry) == Ok(FILE_ENTRY_KIND);
        let name_field = self.bytes(entry + FILE_NAME_FIELD, LONGEST_FILE_NAME);
        let name = name_field.split(|&byte| byte == 0).next();

        is_file && name == Some(SCORE_XML_NAME)
    }

    fn file_at(&mut self, entry: usize) -> Result<FileReader<'_, 'file>, Unreadable> {
        let size = self.number_at(entry + FILE_SIZE_FIELD)?;

        if size > LARGEST_SCORE_XML {
            return Err(Unreadable);
        }

        Ok(FileReader {
            file_system: self,
            sector_list: entry + SECTOR_LIST_FIELD,
            unread: size,
            sector: Vec::new(),
            read_in_sector: 0,
        })
    }

    fn number_at(&mut self, at: usize) -> Result<usize, Unreadable> {
        let Ok(little_endian) = self.bytes(at, NUMBER).try_into() else {
            return Err(Unreadable);
        };

        number_from(little_endian)
    }

    fn bytes(&mut self, at_in_file_system: usize, count: usize) -> &[u8] {
        self.unpacked_bytes(
            at_in_file_system.saturating_add(CONTAINER_NAME_LENGTH),
            count,
        )
    }

    fn unpacked_bytes(&mut self, at: usize, count: usize) -> &[u8] {
        let end = at.saturating_add(count).min(self.length);
        while self.unpacked.len() < end && unpacks_more(&mut self.bits, &mut self.unpacked) {}
        let available = end.min(self.unpacked.len());

        self.unpacked.get(at..available).unwrap_or_default()
    }
}

#[derive(Debug)]
struct FileReader<'system, 'file> {
    file_system: &'system mut FileSystem<'file>,
    sector_list: usize,
    unread: usize,
    sector: Vec<u8>,
    read_in_sector: usize,
}

impl FileReader<'_, '_> {
    fn next_sector(&mut self) -> Result<Vec<u8>, Unreadable> {
        if self.unread == 0 {
            return Ok(Vec::new());
        }
        let sector = self.file_system.number_at(self.sector_list)?;

        if sector == 0 {
            return Ok(Vec::new());
        }
        self.sector_list += NUMBER;
        let start = sector.saturating_mul(SECTOR);
        let bytes = self.file_system.bytes(start, SECTOR.min(self.unread));

        if bytes.is_empty() {
            return Err(Unreadable);
        }
        self.unread -= bytes.len();

        Ok(bytes.to_vec())
    }
}

impl Read for FileReader<'_, '_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.read_in_sector == self.sector.len() {
            let Ok(sector) = self.next_sector() else {
                return Err(io::ErrorKind::InvalidData.into());
            };
            self.sector = sector;
            self.read_in_sector = 0;
        }
        let unread_in_sector = &self.sector[self.read_in_sector..];
        let count = buffer.len().min(unread_in_sector.len());
        buffer[..count].copy_from_slice(&unread_in_sector[..count]);
        self.read_in_sector += count;

        Ok(count)
    }
}

fn unpacks_more(bits: &mut Bits<'_>, unpacked: &mut Vec<u8>) -> bool {
    let before = unpacked.len();
    let step_complete = unpack_step(bits, unpacked).is_some();

    step_complete && unpacked.len() > before
}

fn unpack_step(bits: &mut Bits<'_>, unpacked: &mut Vec<u8>) -> Option<()> {
    let copies_earlier_bytes = bits.read(1)? == 1;

    if copies_earlier_bytes {
        return copy_earlier_bytes(bits, unpacked);
    }
    let literals = bits.read_reversed(2)?;
    for _ in 0..literals {
        unpacked.push(bits.byte()?);
    }

    Some(())
}

fn copy_earlier_bytes(bits: &mut Bits<'_>, unpacked: &mut Vec<u8>) -> Option<()> {
    let word_size = bits.read(4)?;
    let distance = bits.read_reversed(word_size)?;
    let size = bits.read_reversed(word_size)?;
    let start = unpacked.len().checked_sub(distance)?;
    unpacked.extend_from_within(start..start + size.min(distance));

    Some(())
}

#[derive(Debug)]
struct Bits<'file> {
    compressed: &'file [u8],
    position: usize,
}

impl<'file> Bits<'file> {
    const fn over(compressed: &'file [u8]) -> Self {
        Self {
            compressed,
            position: 0,
        }
    }

    fn read(&mut self, count: usize) -> Option<usize> {
        let mut value = 0;
        for _ in 0..count {
            value = value << 1 | self.bit()?;
        }

        Some(value)
    }

    fn read_reversed(&mut self, count: usize) -> Option<usize> {
        let mut value = 0;
        for index in 0..count {
            value |= self.bit()? << index;
        }

        Some(value)
    }

    fn byte(&mut self) -> Option<u8> {
        u8::try_from(self.read(8)?).ok()
    }

    fn bit(&mut self) -> Option<usize> {
        let byte = self.compressed.get(self.position / 8)?;
        let bit = byte >> (7 - self.position % 8) & 1;
        self.position += 1;

        Some(usize::from(bit))
    }
}

fn number_from(little_endian: [u8; NUMBER]) -> Result<usize, Unreadable> {
    usize::try_from(i32::from_le_bytes(little_endian)).map_err(|_| Unreadable)
}

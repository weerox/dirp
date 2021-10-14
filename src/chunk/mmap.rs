use std::io::Read;

use crate::endian::{Endian, Endianness};

pub struct MemoryMap {
    entries: Vec<MemoryMapEntry>,
}

impl MemoryMap {
    pub fn entries(&self) -> &Vec<MemoryMapEntry> {
        &self.entries
    }
}

pub struct MemoryMapEntry {
    chunk: String,
    size: u32,
    offset: u32,
}

impl MemoryMapEntry {
    pub fn chunk(&self) -> &str {
        &self.chunk
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }
}

pub fn read_mmap<R: Read + Endian, E: Endianness>(file: &mut R) -> MemoryMap {
    let mut mmap = [0; 4];
    file.read_bytes::<E>(&mut mmap);
    if mmap != [b'm', b'm', b'a', b'p'] {
        panic!("Chunk header was incorrect");
    }

    let _size = file.read_u32::<E>();

    file.read_u16::<E>();
    file.read_u16::<E>();

    let _chunk_count_max = file.read_u32::<E>();
    let chunk_count_used = file.read_u32::<E>();

    file.read_u32::<E>();
    file.read_u32::<E>();
    file.read_u32::<E>();

    let mut entries = Vec::new();

    for _ in 0..chunk_count_used {
        let mut chunk = [0; 4];
        file.read_bytes::<E>(&mut chunk);
        let chunk = String::from_utf8(Vec::from(chunk)).unwrap();

        let size = file.read_u32::<E>();

        let offset = file.read_u32::<E>();

        file.read_u16::<E>();
        file.read_u16::<E>();
        file.read_u16::<E>();

        let entry = MemoryMapEntry {
            chunk: chunk,
            size: size,
            offset: offset,
        };

        entries.push(entry);
    }

    MemoryMap {
        entries: entries,
    }
}

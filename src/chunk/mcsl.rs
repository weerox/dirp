use std::io::Read;

use crate::endian::{Endian, Endianness, BigEndian};

pub struct MovieCastList {
    entries: Vec<Cast>,
}

impl MovieCastList {
    pub fn entries(&self) -> &Vec<Cast> {
        &self.entries
    }
}

pub struct Cast {
    name: String,
    path: String,
    min: u8,
    max: u8,
    member_count: u16,
    id: u32,
}

impl Cast {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

pub fn read_mcsl<R: Read + Endian, E: Endianness>(file: &mut R) -> MovieCastList {
    let mut mcsl = [0; 4];
    file.read_bytes::<E>(&mut mcsl);
    if mcsl != [b'M', b'C', b's', b'L'] {
        panic!("Chunk header was incorrect");
    }

    let _size = file.read_u32::<E>();

    file.read_u32::<BigEndian>();

    let count = file.read_u32::<BigEndian>();

    file.read_u16::<BigEndian>();

    let x = file.read_u32::<BigEndian>();
    for _ in 0..x {
        file.read_u32::<BigEndian>();
    }

    file.read_u32::<BigEndian>();

    let mut entries = Vec::new();

    for _ in 0..count {
        let len = file.read_u8::<BigEndian>();
        let mut name = vec![0; len as usize];
        file.read_bytes::<BigEndian>(&mut name);
        let name = String::from_utf8(name).unwrap();
        file.read_u8::<BigEndian>();

        let len = file.read_u8::<BigEndian>();
        let mut path = vec![0; len as usize];
        file.read_bytes::<BigEndian>(&mut path);
        let path = String::from_utf8(path).unwrap();
        file.read_u8::<BigEndian>();

        if !path.is_empty() {
            file.read_u8::<BigEndian>();
        }

        let min = file.read_u8::<BigEndian>();
        let max = file.read_u8::<BigEndian>();

        let member_count = file.read_u16::<BigEndian>();

        let id = file.read_u32::<BigEndian>();

        entries.push(Cast {
            name: name,
            path: path,
            min: min,
            max: max,
            member_count: member_count,
            id: id,
        });
    }

    MovieCastList {
        entries: entries,
    }
}

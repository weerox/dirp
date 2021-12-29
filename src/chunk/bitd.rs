use std::io::Read;

use crate::endian::{Endian, Endianness, BigEndian};

pub struct BitmapData {
    data: Vec<u8>,
}

impl BitmapData {
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

pub fn read_bitd<R: Read + Endian, E: Endianness>(file: &mut R) -> BitmapData {
    let mut mmap = [0; 4];
    file.read_bytes::<E>(&mut mmap);
    if mmap != [b'B', b'I', b'T', b'D'] {
        panic!("Chunk header was incorrect");
    }

    let size = file.read_u32::<E>();

    let mut data = vec![0; size as usize];
    file.read_bytes::<BigEndian>(&mut data);

    BitmapData {
        data: data,
    }
}

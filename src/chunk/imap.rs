use std::io::Read;

use crate::endian::{Endian, Endianness};

pub struct InitialMap {
    memory_map_offset: u32,
}

pub fn read_imap<R: Read + Endian, E: Endianness>(file: &mut R) -> InitialMap {
    let mut imap = [0; 4];
    file.read_bytes::<E>(&mut imap);
    if imap != [b'i', b'm', b'a', b'p'] {
        panic!("Chunk header was incorrect");
    }

    let _size = file.read_u32::<E>();

    let count = file.read_u32::<E>();

    debug_assert_eq!(count, 1);

    let offset = file.read_u32::<E>();

    InitialMap {
        memory_map_offset: offset,
    }
}

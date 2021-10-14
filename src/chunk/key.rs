use std::io::Read;

use crate::endian::{Endian, Endianness};

pub struct KeyTable {
    keys: Vec<Key>,
}

pub struct Key {
    owned: u32,
    owner: u32,
    chunk: String,
}

pub fn read_key<R: Read + Endian, E: Endianness>(file: &mut R) -> KeyTable {
    let mut key = [0; 4];
    file.read_bytes::<E>(&mut key);
    if key != [b'K', b'E', b'Y', b'*'] {
        panic!("Chunk header was incorrect");
    }

    let _size = file.read_u32::<E>();

    file.read_u16::<E>();
    file.read_u16::<E>();

    // NOTE Schokky parse 'max' number of keys instead of 'used'
    let _max_key_count = file.read_u32::<E>();
    let used_key_count = file.read_u32::<E>();

    let mut keys = Vec::new();

    for _ in 0..used_key_count {
        let owned = file.read_u32::<E>();
        let owner = file.read_u32::<E>();
        let mut chunk = [0; 4];
        file.read_bytes::<E>(&mut chunk);
        let chunk = String::from_utf8(Vec::from(chunk)).unwrap();

        keys.push(Key {
            owned: owned,
            owner: owner,
            chunk: chunk,
        });
    }

    KeyTable {
        keys: keys,
    }
}

use std::io::Read;

use crate::endian::{Endian, Endianness, BigEndian};

pub struct CastTable {
    members: Vec<Member>,
}

type Member = u32;

pub fn read_cas<R: Read + Endian, E: Endianness>(file: &mut R) -> CastTable {
    let mut key = [0; 4];
    file.read_bytes::<E>(&mut key);
    if key != [b'C', b'A', b'S', b'*'] {
        panic!("Chunk header was incorrect");
    }

    let size = file.read_u32::<E>();

    eprintln!("Size of CAS*: {}", size);

    let member_count = size / std::mem::size_of::<u32>() as u32;

    let mut members = Vec::new();

    for _ in 0..member_count {
        let member = file.read_u32::<BigEndian>();
        eprintln!("Cast member: {:08x}", member);
        members.push(member);
    }

    CastTable {
        members: members,
    }
}

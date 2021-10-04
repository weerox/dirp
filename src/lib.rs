use std::fs::File;
use std::io::Read;
use std::path::Path;

mod chunk;

mod endian;

use chunk::Chunk;

use chunk::rifx::Header;
use chunk::rifx::Endianness;

use endian::{BigEndian, LittleEndian};

pub struct DirectorFile {
    chunks: Vec<Chunk>,
}

impl DirectorFile {
    pub fn new<P: AsRef<Path>>(file: P) -> DirectorFile {
        let mut file = File::open(file.as_ref()).unwrap();

        let mut df = DirectorFile {
            chunks: Vec::new(),
        };

        let header = chunk::rifx::read_rifx(&mut file);

        df.chunks.push(Chunk::Header(header));

        match df.header().endian() {
            Endianness::Big => df.read_chunks::<File, BigEndian>(&mut file),
            Endianness::Little => df.read_chunks::<File, LittleEndian>(&mut file),
        }

        df
    }

    // A helper function to make it easier to use the correct endianness.
    fn read_chunks<R: Read, E: endian::Endianness>(&mut self, file: &mut R) {
    }

    pub fn header(&self) -> &Header {
        let chunk = self.chunks.iter().find(|c| if let Chunk::Header(h) = c {
            true
        } else {
            false
        }).unwrap();

        match chunk {
            Chunk::Header(h) => h,
            _ => unreachable!(),
        }
    }
}

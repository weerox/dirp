use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

mod chunk;

mod endian;

use chunk::Chunk;

use chunk::rifx::Header;
use chunk::rifx::Endianness;

use chunk::imap;
use chunk::imap::InitialMap;

use chunk::mmap;
use chunk::mmap::MemoryMap;

use chunk::key;
use chunk::key::KeyTable;

use chunk::mcsl;
use chunk::mcsl::MovieCastList;

use chunk::cas;
use chunk::cas::CastTable;

use endian::{BigEndian, LittleEndian};

pub struct DirectorFile {
    chunks: Vec<Chunk>,
}

impl DirectorFile {
    // Read the chunks RIFX -> imap -> mmap -> KEY*
    pub fn base<P: AsRef<Path>>(file: P) -> io::Result<DirectorFile> {
        let mut file = File::open(file.as_ref())?;

        let mut df = DirectorFile {
            chunks: Vec::new(),
        };

        let header = chunk::rifx::read_rifx(&mut file);

        df.chunks.push(Chunk::Header(header));

        match df.header().endian() {
            Endianness::Big => df.read_base_chunks::<File, BigEndian>(&mut file),
            Endianness::Little => df.read_base_chunks::<File, LittleEndian>(&mut file),
        }

        Ok(df)
    }

    // Read a dir/dxr file
    pub fn new<P: AsRef<Path>>(file: P) -> io::Result<DirectorFile> {
        let mut base = DirectorFile::base(file.as_ref())?;

        let mut file = File::open(file.as_ref())?;

        match base.header().endian() {
            Endianness::Big => base.read_chunks::<File, BigEndian>(&mut file),
            Endianness::Little => base.read_chunks::<File, LittleEndian>(&mut file),
        }

        Ok(base)
    }

    // A helper function to make it easier to use the correct endianness.
    fn read_base_chunks<R: Read + Seek, E: endian::Endianness>(&mut self, file: &mut R) {
        let imap = imap::read_imap::<R, E>(file);
        self.chunks.push(Chunk::InitialMap(imap));

        let imap = self.imap();

        file.seek(SeekFrom::Start(imap.mmap_offset() as u64)).unwrap();

        let mmap = mmap::read_mmap::<R, E>(file);
        self.chunks.push(Chunk::MemoryMap(mmap));

        let mmap = self.mmap();
        let entries = mmap.entries();

        let key_offset = entries.get(3).unwrap().offset();

        file.seek(SeekFrom::Start(key_offset as u64)).unwrap();

        let key = key::read_key::<R, E>(file);
        self.chunks.push(Chunk::KeyTable(key));
    }

    // Read dir/dxr chunks. The DirectorFile struct passed here must already
    // have parsed the base chunks.
    fn read_chunks<R: Read + Seek, E: endian::Endianness>(&mut self, file: &mut R) {
        let mmap_entries = self.mmap().entries();
        let key = self.key();
        let mcsl_offset = mmap_entries.get(
            key.lookup(0x400, "MCsL".to_string()).unwrap() as usize
        ).unwrap().offset();

        file.seek(SeekFrom::Start(mcsl_offset as u64)).unwrap();

        let mcsl = mcsl::read_mcsl::<R, E>(file);
        self.chunks.push(Chunk::MovieCastList(mcsl));

        let mcsl = self.mcsl().clone();
        for entry in mcsl.entries() {
            if entry.name() == "Internal" {
                continue;
            }

            eprintln!("Parsing cast file {}", entry.name());

            let path = format!("{}.cxt", entry.name());
            let cast = DirectorFile::base(&path);
            let cast = if let Ok(cast) = cast {
                cast
            } else {
                // An error was returned when creating
                // the DirectorFile, so we will skip reading it.
                eprintln!("Couldn't parse file, skipping...");
                continue
            };

            let mut cast_file = File::open(&path).unwrap();

            let key = cast.key();

            // Do a lookup for the id of the CAS* chunk
            let cas_id = key.lookup(0x400, "CAS*".to_string()).unwrap();
            let cas_offset = cast.mmap().entries().get(cas_id as usize).unwrap().offset();

            cast_file.seek(SeekFrom::Start(cas_offset as u64)).unwrap();

            let cas = cas::read_cas::<File, E>(&mut cast_file);
            self.chunks.push(Chunk::CastTable(cas));
        }
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

    pub fn imap(&self) -> &InitialMap {
        let chunk = self.chunks.iter().find(|c| if let Chunk::InitialMap(i) = c {
            true
        } else {
            false
        }).unwrap();

        match chunk {
            Chunk::InitialMap(i) => i,
            _ => unreachable!(),
        }
    }

    pub fn mmap(&self) -> &MemoryMap {
        let chunk = self.chunks.iter().find(|c| if let Chunk::MemoryMap(m) = c {
            true
        } else {
            false
        }).unwrap();

        match chunk {
            Chunk::MemoryMap(m) => m,
            _ => unreachable!(),
        }
    }

    pub fn key(&self) -> &KeyTable {
        let chunk = self.chunks.iter().find(|c| if let Chunk::KeyTable(k) = c {
            true
        } else {
            false
        }).unwrap();

        match chunk {
            Chunk::KeyTable(k) => k,
            _ => unreachable!(),
        }
    }

    pub fn mcsl(&self) -> &MovieCastList {
        let chunk = self.chunks.iter().find(|c| if let Chunk::MovieCastList(m) = c {
            true
        } else {
            false
        }).unwrap();

        match chunk {
            Chunk::MovieCastList(m) => m,
            _ => unreachable!(),
        }
    }
}

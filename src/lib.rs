use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::collections::VecDeque;

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

use chunk::cast;
use chunk::cast::CastProperties;
use chunk::cast::CastProperty;
use chunk::cast::CastKind;

use chunk::bitd;
use chunk::bitd::BitmapData;

use endian::{BigEndian, LittleEndian};

pub struct DirectorFile {
    header: Header,
    imap: InitialMap,
    mmap: MemoryMap,
    key: KeyTable,
}

impl DirectorFile {
    // Read the chunks RIFX -> imap -> mmap -> KEY*
    pub fn base<P: AsRef<Path>>(file: P) -> io::Result<DirectorFile> {
        let mut file = File::open(file.as_ref())?;

        let header = chunk::rifx::read_rifx(&mut file);

        let (imap, mmap, key) = match header.endian() {
            Endianness::Big => read_base_chunks::<File, BigEndian>(&mut file),
            Endianness::Little => read_base_chunks::<File, LittleEndian>(&mut file),
        };

        let df = DirectorFile {
            header: header,
            imap: imap,
            mmap: mmap,
            key: key,
        };

        Ok(df)
    }

    // Read a dir/dxr file
    pub fn new<P: AsRef<Path>>(file: P) -> io::Result<DirectorFile> {
        let mut base = DirectorFile::base(file.as_ref())?;

        let mut file = File::open(file.as_ref())?;

        match base.header().endian() {
            Endianness::Big => read_chunks::<File, BigEndian>(&mut base, &mut file),
            Endianness::Little => read_chunks::<File, LittleEndian>(&mut base, &mut file),
        }

        Ok(base)
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn imap(&self) -> &InitialMap {
        &self.imap
    }

    pub fn mmap(&self) -> &MemoryMap {
        &self.mmap
    }

    pub fn key(&self) -> &KeyTable {
        &self.key
    }
}

// A helper function to make it easier to use the correct endianness.
fn read_base_chunks<R: Read + Seek, E: endian::Endianness>(file: &mut R) -> (InitialMap, MemoryMap, KeyTable){
    let imap = imap::read_imap::<R, E>(file);

    file.seek(SeekFrom::Start(imap.mmap_offset() as u64)).unwrap();

    let mmap = mmap::read_mmap::<R, E>(file);

    let entries = mmap.entries();

    let key_offset = entries.get(3).unwrap().offset();

    file.seek(SeekFrom::Start(key_offset as u64)).unwrap();

    let key = key::read_key::<R, E>(file);

    (imap, mmap, key)
}

// Read dir/dxr chunks. The DirectorFile struct passed here must already
// have parsed the base chunks.
fn read_chunks<R: Read + Seek, E: endian::Endianness>(df: &mut DirectorFile, file: &mut R) {
    let mmap_entries = df.mmap().entries();
    let key = df.key();
    let mcsl_offset = mmap_entries.get(
        key.lookup(0x400, "MCsL".to_string()).unwrap() as usize
    ).unwrap().offset();

    file.seek(SeekFrom::Start(mcsl_offset as u64)).unwrap();

    let mcsl = mcsl::read_mcsl::<R, E>(file);

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

        for &member in cas.members() {
            if member == 0 {
                continue;
            }

            let member_offset = cast.mmap().entries().get(member as usize).unwrap().offset();

            cast_file.seek(SeekFrom::Start(member_offset as u64)).unwrap();

            let cast_properties = cast::read_cast::<File, E>(&mut cast_file);

            match cast_properties.kind() {
                CastKind::Bitmap => {
                    // Bitmaps own the BITD chunk
                    let id = match cast.key().lookup(member, "BITD".to_string()) {
                        Some(x) => x,
                        // TODO For some reason, some of the bitmap casts
                        // doesn't own a BITD chunk...?
                        None => continue,
                    };
                    let offset = cast.mmap().entries().get(id as usize).unwrap().offset();

                    cast_file.seek(SeekFrom::Start(offset as u64)).unwrap();

                    let data = bitd::read_bitd::<File, E>(&mut cast_file);

                    let &depth = cast_properties.properties()
                        .get(&CastProperty::BitmapDepth).unwrap()
                        .downcast_ref::<usize>().unwrap();

                    // The parser can only hande a bit depth of 32.
                    if depth == 32 {
                        let bitmap = parse_bitmap_data(&cast_properties, data);
                    } else {
                        eprintln!("Can only handle bitmaps with bit depth of 32");
                    }
                },
                _ => {
                    eprintln!("This cast type is not supported, skipping")
                }
            }
        }
    }
}

// NOTE We assume that the bit depth is 32
fn parse_bitmap_data(properties: &CastProperties, data: BitmapData) -> Vec<Vec<[u8; 4]>> {
    let &width = properties.properties()
        .get(&CastProperty::BitmapWidth).unwrap()
        .downcast_ref::<usize>().unwrap();
    let &height = properties.properties()
        .get(&CastProperty::BitmapHeight).unwrap()
        .downcast_ref::<usize>().unwrap();

    let mut bitmap = vec![vec![[0u8, 0u8, 0u8, 255u8]; width]; height];

    let mut x = 0;
    let mut y = 0;
    let mut c = 0;

    let mut data = VecDeque::from(data.data().clone());

    while !data.is_empty() {
        let len = data.pop_front().unwrap();

        // NOTE This value is from Shockky. MrBrax use >= 129
        if len >= 128 {
            let len = 257 - len as u16;
            let len = len as u8;

            let b = data.pop_front().unwrap();

            for _ in 0..len {
                bitmap[y][x][c] = b;

                x += 1;
                if x >= width {
                    c += 1;
                    c %= 4;

                    x = 0;

                    if c == 0 {
                        y += 1;
                    }
                }

                if y >= height {
                    return bitmap;
                }
            }
        } else {
            let len = len + 1;

            for _ in 0..len {
                let b = data.pop_front().unwrap();

                bitmap[y][x][c] = b;

                x += 1;

                if x >= width {
                    c += 1;
                    c %= 4;

                    x = 0;

                    if c == 0 {
                        y += 1;
                    }
                }

                if y >= height {
                    return bitmap;
                }
            }
        }
    }

    bitmap
}

use std::io::Read;

use std::collections::HashMap;

use std::any::Any;

use crate::endian::{Endian, Endianness, BigEndian};

pub struct CastProperties {
    kind: CastKind,
    // TODO It would be nice if we could change this to a HashSet of enums,
    // where only the enum type and not the value is part of the hash.
    properties: HashMap<CastProperty, Box<dyn Any>>,
}

impl CastProperties {
    pub fn kind(&self) -> CastKind {
        self.kind
    }

    pub fn properties(&self) -> &HashMap<CastProperty, Box<dyn Any>> {
        &self.properties
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum CastProperty {
    Name = 1,          // String
    XtraName = 10,     // String
    BitmapWidth = 22,  // usize
    BitmapHeight = 23, // usize
    BitmapDepth = 24,  // usize
}

#[derive(Copy, Clone)]
pub enum CastKind {
    Bitmap = 1,
    FilmLoop,
    StyledText,
    Palette,
    Picture,
    Sound,
    Button,
    Shape,
    Movie,
    DigitalVideo,
    Script,
    Text,
    OLE,
    Transition,
    Xtra,
}

pub fn read_cast<R: Read + Endian, E: Endianness>(file: &mut R) -> CastProperties {
    let mut key = [0; 4];
    file.read_bytes::<E>(&mut key);
    if key != [b'C', b'A', b'S', b't'] {
        panic!("Chunk header was incorrect");
    }

    let _size = file.read_u32::<E>();
    let mut read = 0;

    let kind = file.read_u32::<BigEndian>();
    read += 4;
    eprintln!("cast type: {}", kind);
    let kind = match kind {
        1 =>  CastKind::Bitmap,
        2 =>  CastKind::FilmLoop,
        3 =>  CastKind::StyledText,
        4 =>  CastKind::Palette,
        5 =>  CastKind::Picture,
        6 =>  CastKind::Sound,
        7 =>  CastKind::Button,
        8 =>  CastKind::Shape,
        9 =>  CastKind::Movie,
        10 => CastKind::DigitalVideo,
        11 => CastKind::Script,
        12 => CastKind::Text,
        13 => CastKind::OLE,
        14 => CastKind::Transition,
        15 => CastKind::Xtra,
        _ => panic!("Unknown cast type: {}", kind),
    };

    // This seem to be the size of the general properties.
    let _a = file.read_u32::<BigEndian>();
    read += 4;
    // This seem to be the size of the type specific properties.
    let _b = file.read_u32::<BigEndian>();
    read += 4;

    eprintln!("{} {}", _a, _b);

    // NOTE the _a size is from here

    let _c = file.read_u32::<BigEndian>();
    read += 4;
    let _d = file.read_u32::<BigEndian>();
    read += 4;
    let _e = file.read_u32::<BigEndian>();
    read += 4;
    let _f = file.read_u32::<BigEndian>();
    read += 4;
    let _g = file.read_u32::<BigEndian>();
    read += 4;

    let offset_count = file.read_u16::<BigEndian>();
    read += 2;

    let mut offsets: Vec<usize> = Vec::new();

    for _ in 0..(offset_count + 1) {
        let offset = file.read_u32::<BigEndian>() as usize;
        read += 4;
        offsets.push(offset);
    }

    let mut properties = HashMap::new();

    for i in 0..offset_count as usize {
        let len = offsets[i + 1] - offsets[i];
        if len == 0 {
            continue;
        }

        let prop = read_property(file, i, len);
        read += len;
        if let Some((name, value)) = prop {
            properties.insert(name, value);
        }
    }

    // NOTE The _b size is from here

    // TODO Parse the properties for the specific type

    match kind {
        CastKind::Bitmap => {
            file.read_u16::<BigEndian>();

            let top = file.read_u16::<BigEndian>();
            let left = file.read_u16::<BigEndian>();
            let bottom = file.read_u16::<BigEndian>();
            let right = file.read_u16::<BigEndian>();

            eprintln!("{} {} {} {}", top, left, bottom, right);

            let height = bottom - top;
            let width = right - left;

            file.read_u32::<BigEndian>();
            file.read_u32::<BigEndian>();

            let point_x = file.read_u16::<BigEndian>();
            let point_y = file.read_u16::<BigEndian>();

            eprintln!("{} {}", point_x, point_y);

            let _a = file.read_u8();
            let bit_depth = file.read_u8();

            eprintln!("{} {}", _a, bit_depth);

            // _c is always -1 and _d is always -101
            let _c = file.read_u16::<BigEndian>() as i16;
            let _d = file.read_u16::<BigEndian>() as i16;


            properties.insert(
                CastProperty::BitmapWidth, Box::new(width as usize)
            );
            properties.insert(
                CastProperty::BitmapHeight, Box::new(height as usize)
            );
            properties.insert(
                CastProperty::BitmapDepth, Box::new(bit_depth as usize)
            );
        },
        _ => {
            eprintln!("Unsupported cast type, skipping type specific properties...");
        },
    }

    eprintln!();

    CastProperties {
        kind: kind,
        properties: properties,
    }
}

fn read_property<R: Read + Endian>(
    file: &mut R,
    index: usize,
    len: usize
) -> Option<(CastProperty, Box<dyn Any>)> {
    match index {
        1 => {
            let str_len = file.read_u8() as usize;

            // Make sure that we don't read more bytes than
            // the given length of the property.
            let str_len = if str_len > len - 1 {
                len - 1
            } else {
                str_len
            };

            let mut name = vec![0; str_len];
            file.read_bytes::<BigEndian>(&mut name);
            let name = String::from_utf8(name).unwrap();
            //let _null = file.read_u8();

            eprintln!("name: {}", name);

            Some((CastProperty::Name, Box::new(name)))
        },
        10 => {
            // NOTE This will be terminated by a NULL byte,
            // which we don't strip.
            let mut name = vec![0; len];
            file.read_bytes::<BigEndian>(&mut name);
            let name = String::from_utf8(name).unwrap();
            //let _null = file.read_u8();

            eprintln!("name: {}", name);

            Some((CastProperty::XtraName, Box::new(name)))
        },
        i => {
            let mut scrap = vec![0; len];
            file.read_bytes::<BigEndian>(&mut scrap);
            eprintln!("Can't parse cast property with index {}", i);
            None
        }
    }
}

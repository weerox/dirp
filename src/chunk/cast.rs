use std::io::Read;

use std::collections::HashMap;

use crate::endian::{Endian, Endianness, BigEndian};

pub struct CastProperties {
    t: CastType,
    // TODO It would be nice if we could change this to a HashSet of enums,
    // where only the enum type and not the value is part of the hash.
    properties: HashMap<CastPropertyName, CastPropertyValue>,
}

#[derive(PartialEq, Eq, Hash)]
pub enum CastPropertyName {
    Name = 1,
    XtraName = 10,
    BitmapWidth = 22,
    BitmapHeight = 23,
    BitmapDepth = 24,
}

#[derive(PartialEq, Eq, Hash)]
pub enum CastPropertyValue {
    Name(String),
    XtraName(String),
    BitmapWidth(usize),
    BitmapHeight(usize),
    BitmapDepth(usize),
}

pub enum CastType {
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

    let t = file.read_u32::<BigEndian>();
    read += 4;
    eprintln!("cast type: {}", t);
    let t = match t {
        1 =>  CastType::Bitmap,
        2 =>  CastType::FilmLoop,
        3 =>  CastType::StyledText,
        4 =>  CastType::Palette,
        5 =>  CastType::Picture,
        6 =>  CastType::Sound,
        7 =>  CastType::Button,
        8 =>  CastType::Shape,
        9 =>  CastType::Movie,
        10 => CastType::DigitalVideo,
        11 => CastType::Script,
        12 => CastType::Text,
        13 => CastType::OLE,
        14 => CastType::Transition,
        15 => CastType::Xtra,
        _ => panic!("Unknown cast type: {}", t),
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

    match t {
        CastType::Bitmap => {
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

            let _a = file.read_u8::<BigEndian>();
            let bit_depth = file.read_u8::<BigEndian>();

            eprintln!("{} {}", _a, bit_depth);

            // _c is always -1 and _d is always -101
            let _c = file.read_u16::<BigEndian>() as i16;
            let _d = file.read_u16::<BigEndian>() as i16;

            properties.insert(
                CastPropertyName::BitmapWidth, CastPropertyValue::BitmapWidth(width as usize)
            );
            properties.insert(
                CastPropertyName::BitmapHeight, CastPropertyValue::BitmapHeight(height as usize)
            );
            properties.insert(
                CastPropertyName::BitmapDepth, CastPropertyValue::BitmapDepth(bit_depth as usize)
            );
        },
        _ => {
            eprintln!("Unsupported cast type, skipping type specific properties...");
        },
    }

    eprintln!();

    CastProperties {
        t: t,
        properties: properties,
    }
}

fn read_property<R: Read + Endian>(
    file: &mut R,
    index: usize,
    len: usize
) -> Option<(CastPropertyName, CastPropertyValue)> {
    match index {
        1 => {
            let str_len = file.read_u8::<BigEndian>() as usize;

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
            //let _null = file.read_u8::<BigEndian>();

            eprintln!("name: {}", name);

            Some((CastPropertyName::Name, CastPropertyValue::Name(name)))
        },
        10 => {
            // NOTE This will be terminated by a NULL byte,
            // which we don't strip.
            let mut name = vec![0; len];
            file.read_bytes::<BigEndian>(&mut name);
            let name = String::from_utf8(name).unwrap();
            //let _null = file.read_u8::<BigEndian>();

            eprintln!("name: {}", name);

            Some((CastPropertyName::XtraName, CastPropertyValue::XtraName(name)))
        },
        i => {
            let mut scrap = vec![0; len];
            file.read_bytes::<BigEndian>(&mut scrap);
            eprintln!("Can't parse cast property with index {}", i);
            None
        }
    }
}

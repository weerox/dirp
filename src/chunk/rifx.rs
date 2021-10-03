use std::io::Read;

pub struct Header {
    endian: Endianness,
    size: u32,
    codec: String,
}

#[derive(Copy, Clone)]
pub enum Endianness {
    Big, Little,
}

impl Header {
    pub fn endian(&self) -> Endianness {
        self.endian
    }
}

pub fn read_rifx<R: Read>(file: &mut R) -> Header {
    let mut rifx = [0; 4];
    file.read(&mut rifx).unwrap();

    let endian = if rifx == [b'R', b'I', b'F', b'X'] {
        Endianness::Big
    } else if rifx == [b'X', b'F', b'I', b'R'] {
        Endianness::Little
    } else {
        panic!("Header did not start with RIFX or XFIR");
    };

    let mut size = [0; 4];
    file.read(&mut size).unwrap();

    let size = match endian {
        Endianness::Big    => u32::from_be_bytes(size),
        Endianness::Little => u32::from_le_bytes(size),
    };

    let mut codec = [0; 4];
    file.read(&mut codec).unwrap();

    let codec = match endian {
        Endianness::Big    => {
            String::from_utf8(Vec::from(codec)).unwrap()
        },
        Endianness::Little => {
            let mut v = Vec::from(codec);
            v.reverse();
            String::from_utf8(v).unwrap()
        },
    };

    Header {
        endian: endian,
        size: size,
        codec: codec,
    }
}

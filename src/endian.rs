use std::io::Read;

pub struct BigEndian;
pub struct LittleEndian;

pub trait Endianness {
    fn is_big_endian() -> bool;
}

impl Endianness for BigEndian {
    fn is_big_endian() -> bool {
        true
    }
}

impl Endianness for LittleEndian {
    fn is_big_endian() -> bool {
        false
    }
}

pub trait Endian: Read {
    fn read_bytes<E: Endianness>(&mut self, bytes: &mut [u8]) {
        self.read(bytes).unwrap();

        if !E::is_big_endian() {
            bytes.reverse();
        }
    }

    fn read_u8(&mut self) -> u8 {
        let mut bytes = [0; 1];
        self.read(&mut bytes).unwrap();
        // a byte is the same in BE and LE
        u8::from_be_bytes(bytes)
    }

    fn read_u16<E: Endianness>(&mut self) -> u16 {
        let mut bytes = [0; 2];
        self.read(&mut bytes).unwrap();

        if E::is_big_endian() {
            u16::from_be_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        }
    }

    fn read_u32<E: Endianness>(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.read(&mut bytes).unwrap();

        if E::is_big_endian() {
            u32::from_be_bytes(bytes)
        } else {
            u32::from_le_bytes(bytes)
        }
    }

    fn read_u64<E: Endianness>(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.read(&mut bytes).unwrap();

        if E::is_big_endian() {
            u64::from_be_bytes(bytes)
        } else {
            u64::from_le_bytes(bytes)
        }
    }
}

impl<R: Read> Endian for R {}

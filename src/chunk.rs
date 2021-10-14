pub mod rifx;
pub mod imap;
pub mod mmap;
pub mod key;

use rifx::Header;
use imap::InitialMap;
use mmap::MemoryMap;
use key::KeyTable;

pub enum Chunk {
    Header(Header),
    InitialMap(InitialMap),
    MemoryMap(MemoryMap),
    KeyTable(KeyTable),
}

pub mod rifx;
pub mod imap;
pub mod mmap;

use rifx::Header;
use imap::InitialMap;
use mmap::MemoryMap;

pub enum Chunk {
    Header(Header),
    InitialMap(InitialMap),
    MemoryMap(MemoryMap),
}

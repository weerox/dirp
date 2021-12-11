pub mod rifx;
pub mod imap;
pub mod mmap;
pub mod key;
pub mod mcsl;
pub mod cas;

use rifx::Header;
use imap::InitialMap;
use mmap::MemoryMap;
use key::KeyTable;
use mcsl::MovieCastList;
use cas::CastTable;

pub enum Chunk {
    Header(Header),
    InitialMap(InitialMap),
    MemoryMap(MemoryMap),
    KeyTable(KeyTable),
    MovieCastList(MovieCastList),
    CastTable(CastTable),
}

pub mod rifx;
pub mod imap;
pub mod mmap;
pub mod key;
pub mod mcsl;
pub mod cas;
pub mod cast;
pub mod bitd;

use rifx::Header;
use imap::InitialMap;
use mmap::MemoryMap;
use key::KeyTable;
use mcsl::MovieCastList;
use cas::CastTable;
use cast::CastProperties;
use bitd::BitmapData;

pub enum Chunk {
    Header(Header),
    InitialMap(InitialMap),
    MemoryMap(MemoryMap),
    KeyTable(KeyTable),
    MovieCastList(MovieCastList),
    CastTable(CastTable),
    CastProperties(CastProperties),
    BitmapData(BitmapData),
}

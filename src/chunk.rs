pub mod rifx;
pub mod imap;

use rifx::Header;
use imap::InitialMap;

pub enum Chunk {
    Header(Header),
    InitialMap(InitialMap),
}

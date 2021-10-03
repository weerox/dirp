pub mod rifx;

use rifx::Header;

pub enum Chunk {
    Header(Header),
}

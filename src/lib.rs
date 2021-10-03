use std::fs::File;
use std::path::Path;

mod chunk;

use chunk::Chunk;

pub struct DirectorFile {
    chunks: Vec<Chunk>,
}

impl DirectorFile {
    pub fn new<P: AsRef<Path>>(file: P) -> DirectorFile {
        let mut file = File::open(file.as_ref()).unwrap();
        let mut chunks = Vec::new();

        let header = chunk::rifx::read_rifx(&mut file);

        chunks.push(Chunk::Header(header));

        DirectorFile {
            chunks: chunks,
        }
    }
}

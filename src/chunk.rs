use std::path::{Path, PathBuf};
use std::collections::HashSet;

use filebuffer::FileBuffer;
use sodiumoxide::crypto::hash::sha256;

use crc::crc16;
use error::Result;

#[derive(Eq, PartialEq, Debug, Hash)]
struct Chunk {
    digest: sha256::Digest,
    len: usize,
}

struct ChunksMeta {
    num_chunks: usize,
    total_size: usize,
}

impl Chunk {
    pub fn new(data: &[u8]) -> Chunk {
        Chunk {
            digest: sha256::hash(data),
            len: data.len(),
        }
    }
}

fn split_buffer_into_chunks(
    min_chunk_len: u32,
    target_chunk_len: u32,
    data: &[u8],
    chunks: &mut HashSet<Chunk>
    ) -> ChunksMeta
{
    let mut crc = 1;
    let mut data_slice = data;
    let mut has_more = true;
    let mut meta = ChunksMeta { num_chunks: 0, total_size: 0, };

    while has_more {
        has_more = false;
        let mut split_threshold = 0xffff / target_chunk_len as u16;

        for (i, &b) in data_slice.iter().enumerate() {
            crc = crc16(crc, b);
            if crc < split_threshold && i >= min_chunk_len as usize {
                let (chunk, remainder) = data_slice.split_at(i);
                assert!(data_slice.len() > remainder.len(), "{} > {}", data_slice.len(), remainder.len());

                chunks.insert(Chunk::new(chunk));
                meta.num_chunks += 1;
                meta.total_size += chunk.len();

                data_slice = remainder;
                has_more = data_slice.len() > 0;
                crc = 1;
                break;
            }

            // Increase the splitting probability as the chunk grows larger, to
            // avoid very large chunks due to being unlucky. This also benefits
            // chunk reuse.
            if i >= target_chunk_len as usize {
                split_threshold += 2;
            }
        }
        if !has_more {
            chunks.insert(Chunk::new(data_slice));
            meta.num_chunks += 1;
            meta.total_size += data_slice.len();
        }
    }

    meta
}

/// Split a file into chunks. Mmaps the file.
fn split_file_into_chunks(
    min_chunk_len: u32,
    target_chunk_len: u32,
    path: &Path, chunks: &mut HashSet<Chunk>,
    ) -> Result<ChunksMeta>
{
    let fbuffer = FileBuffer::open(path)?;
    Ok(split_buffer_into_chunks(min_chunk_len, target_chunk_len, &fbuffer[..], chunks))
}

/// Chunk all given files, print statistics.
pub fn split_and_print_stats(
    min_chunk_len: u32,
    target_chunk_len: u32,
    paths: &[PathBuf],
    ) -> Result<()>
{
    let mut chunks = HashSet::new();
    let mut total_size = 0;
    let mut dedup_size = 0;
    let mut overhead = 0;
    for path in paths {
        let meta = split_file_into_chunks(min_chunk_len, target_chunk_len, path.as_ref(), &mut chunks)?;
        total_size += meta.total_size;
        // For the index file, 32 bytes of sha256 and 4 bytes of len per chunk.
        overhead += 36 * meta.num_chunks;
    }
    for chunk in chunks {
        dedup_size += chunk.len;
    }

    println!("total size: {}", total_size);
    println!("dedup size: {}", dedup_size);
    println!("overhead:   {}", overhead);
    println!("ratio:      {:6.2}%", 100.0 * (dedup_size + overhead) as f64 / total_size as f64);

    Ok(())
}

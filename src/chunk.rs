use std::path::Path;

use filebuffer::FileBuffer;
use sodiumoxide::crypto::hash::sha256;

use crc::crc16;
use error::Result;
use util;

const MIN_CHUNK_LEN: usize = 512;
const IDEAL_SIZE: usize = 2048;

fn print_chunk(crc: u16, chunk: &[u8]) {
    let digest = sha256::hash(chunk);
    let mut digest_hex = String::new();
    util::append_hex(&mut digest_hex, digest.as_ref());
    println!("{:3} {:04x} {}", chunk.len(), crc, digest_hex);
}

fn split_buffer_into_chunks(data: &[u8]) {
    let mut crc = 1;
    let mut data_slice = data;
    let mut has_more = true;

    while has_more {
        has_more = false;
        let mut split_threshold = 0xffff / IDEAL_SIZE as u16;

        for (i, &b) in data_slice.iter().enumerate() {
            crc = crc16(crc, b);
            if crc < split_threshold && i >= MIN_CHUNK_LEN {
                let (chunk, remainder) = data_slice.split_at(i);
                print_chunk(crc, chunk);
                assert!(data_slice.len() > remainder.len(), "{} > {}", data_slice.len(), remainder.len());
                data_slice = remainder;
                has_more = data_slice.len() > 0;
                crc = 1;
                break;
            }

            // Increase the splitting probability as the chunk grows larger, to
            // avoid very large chunks due to being unlucky. This also benefits
            // chunk reuse.
            if i >= IDEAL_SIZE {
                split_threshold += 2;
            }
        }
        if !has_more {
            print_chunk(crc, data_slice);
        }
    }
}

/// Split a file into chunks. Mmaps the file.
pub fn split_file_into_chunks(path: &Path) -> Result<()> {
    let fbuffer = FileBuffer::open(path)?;
    split_buffer_into_chunks(&fbuffer[..]);
    Ok(())
}

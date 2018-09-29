use std::path::Path;

use filebuffer::FileBuffer;
use sodiumoxide::crypto::hash::sha256;

use crc::crc16;
use error::Result;
use util;

fn split_buffer_into_chunks(data: &[u8]) {
    let mut crc = 1;
    let mut data_slice = data;
    loop {
        for (i, &b) in data_slice.iter().enumerate() {
            crc = crc16(crc, b);
            println!("{} 0x{:02x} {:04x}", i, b, crc);
            if crc & 0xfff == 0 {
                let (chunk, remainder) = data_slice.split_at(i);
                let digest = sha256::hash(chunk);
                let mut digest_hex = String::new();
                util::append_hex(&mut digest_hex, digest.as_ref());
                println!("{} {} {:04x} {}", i, chunk.len(), crc, digest_hex);
                assert!(data_slice.len() > remainder.len(), "{} > {}", data_slice.len(), remainder.len());
                data_slice = remainder;
                break;
            }
        }
        break;
        if data_slice.len() == 0 {
            break;
        }
    }
}

/// Split a file into chunks. Mmaps the file.
pub fn split_file_into_chunks(path: &Path) -> Result<()> {
    let fbuffer = FileBuffer::open(path)?;
    split_buffer_into_chunks(&fbuffer[..]);
    Ok(())
}

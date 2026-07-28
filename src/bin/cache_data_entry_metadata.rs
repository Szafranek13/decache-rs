//The only parser crate there is for this shit doesn't work and hasn't been updated in the last 9 months. Fuck my faggy ass again I guess >:(


use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

/// Magick number at the beggining of every Chromium cache entry file
const CHROMIUM_CACHE_ENTRY_MAGICK: u32 = u32::from_le_bytes([0x30, 0x5C, 0x72, 0xA7]);

#[derive(Debug)]
struct EntryHeader {
    magick: u32,
    version: u32,
    key_len: u32,
    key_hash: u32,
    stream_sizes: [u32; 4],
}

fn read_u32_le<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("/home/jakub/.cache/chromium/Default/Cache/Cache_Data/0340d4a5082cbe60_0")?;
    //Get magick filetype
    let magick = read_u32_le(&mut file)?;
    let version = read_u32_le(&mut file)?;
    let key_len = read_u32_le(&mut file)?; // TODO: FIX IT, IT HAS TO BE WRONG
    let key_hash = read_u32_le(&mut file)?;

    let mut stream_sizes = [0u32; 4];
    for size in &mut stream_sizes { //TODO: Find out what are those. There's no way size of stream_0 is 5 bytes
        *size = read_u32_le(&mut file)?;
    }

    if magick != CHROMIUM_CACHE_ENTRY_MAGICK {
        return Err("Not a Chromium cache entry file".into());
    }

    let fag = EntryHeader{
        magick,
        version,
        key_len,
        key_hash,
        stream_sizes
    };

    //STREAM 0 STARTS HERE
    //TODO: Parse each stream

    //let mut buf = [0; 100];
    //file.read_exact(&mut buf);
    //let key = String::from_utf8(buf.to_vec())?;
    //println!("{:#?}", key);
    //let file_size = file.seek(SeekFrom::End(0))?;

    Ok(())
}

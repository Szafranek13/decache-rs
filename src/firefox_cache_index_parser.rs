//!This script parses firefox cache index file efficently and easly.
//!
//!I have found no easy way to do that so i have written my own based on this ancient Python script
//!<https://github.com/JamesHabben/FirefoxCache2/blob/master/firefox-cache2-index-parser.py>

use byteorder::{BigEndian, ReadBytesExt};
use std::path::{Path, PathBuf};
use std::{
    fs::File,
    io::{self, BufReader, Read},
};

///Struct representing the header of `index` file
#[derive(Debug)]
struct Header {
    version: u32,
    timestamp: u32,
    dirty: u32,
    kb_written: u32,
}

#[derive(Debug)]
struct Record {
    hash: [u8; 20],
    frecency: u32,
    origin_attrs_hash: u64,
    on_start_time: u16,
    on_stop_time: u16,
    content_type: u8,
    flags: u32,
}

fn read_header(r: &mut BufReader<File>) -> io::Result<Header> {
    Ok(Header {
        version: r.read_u32::<BigEndian>()?,
        timestamp: r.read_u32::<BigEndian>()?,
        dirty: r.read_u32::<BigEndian>()?,
        kb_written: r.read_u32::<BigEndian>()?,
    })
}

fn read_record<R: Read>(r: &mut R) -> io::Result<Record> {
    let mut hash = [0u8; 20];
    r.read_exact(&mut hash)?;

    Ok(Record {
        hash,
        frecency: r.read_u32::<BigEndian>()?,
        origin_attrs_hash: r.read_u64::<BigEndian>()?,
        on_start_time: r.read_u16::<BigEndian>()?,
        on_stop_time: r.read_u16::<BigEndian>()?,
        content_type: r.read_u8()?,
        flags: r.read_u32::<BigEndian>()?,
    })
}

fn hex(hash: &[u8]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

fn get_entries_from_index(index: &str) -> io::Result<Vec<PathBuf>> {
    let file = File::open(index).expect("No such file");
    let mut reader = BufReader::new(file);
    let header = read_header(&mut reader).expect("Could not read header");

    let mut entries_vec: Vec<PathBuf> = Vec::new();

    let entries_path =
        PathBuf::from("/home/jakub/.cache/librewolf/bgvrjjel.default-default/cache2/entries/");
    loop {
        match read_record(&mut reader) {
            Ok(record) => {
                let filename = record
                    .hash
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<String>();
                entries_vec.push(entries_path.join(filename));
                let size_b: u32 = record.flags; // & 0x00FF_FFFF;
                let size_kb = record.flags & 0x00FF_FFFF;
                println!();
                println!("hash: {}", hex(&record.hash).to_uppercase());
                println!("frecency: {}", record.frecency);
                println!("size_b: {}", size_b);
                println!("size_kb: {}", size_kb);
                println!("origin_attrs_hash: {}", record.origin_attrs_hash);
                println!("on_start_time: {}", record.on_start_time);
                println!("on_stop_time: {}", record.on_stop_time);
                println!("content_type: {}", record.content_type);
                println!("flags: 0x{:08X}", record.flags);
            }

            //if EOF reached then all entries has been read
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                break;
            }

            Err(e) => return Err(e),
        }
    }
    Ok(entries_vec)
}

fn main() -> io::Result<()> {
    let entries_files = get_entries_from_index(
        "/home/jakub/.cache/librewolf/bgvrjjel.default-default/cache2/index",
    )?;
    //println!("{:#?}", entries_files);
    //let data = std::fs::read(entries_path.join(filename)).unwrap();

    Ok(())
}

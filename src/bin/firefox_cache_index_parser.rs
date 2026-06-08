//!This script parses firefox cache index file efficently and easly.
//!
//!I have found no easy way to do that so i have written my own based on this ancient Python script
//!<https://github.com/JamesHabben/FirefoxCache2/blob/master/firefox-cache2-index-parser.py>

use std::{fs::File, io::{self, BufReader, Read}};
use byteorder::{BigEndian, ReadBytesExt};

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

fn main() -> io::Result<()> {
	let file = File::open("/home/jakub/.cache/librewolf/bgvrjjel.default-default/cache2/index").expect("No such file");

 	let mut reader = BufReader::new(file);
	let header = read_header(&mut reader).expect("Could not read header");

    loop {
        match read_record(&mut reader) {
            Ok(record) => {
                let size_kb = record.flags & 0x00FF_FFFF;

                println!();
                println!("hash: {}", hex(&record.hash));
                println!("frecency: {}", record.frecency);
                println!("size_kb: {}", size_kb);
                println!("content_type: {}", record.content_type);
                println!("flags: 0x{:08x}", record.flags);
            }

            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                break;
            }

            Err(e) => return Err(e),
        }
    }

    Ok(())
}

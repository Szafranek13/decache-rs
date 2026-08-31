use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

const INITIAL_MAGIC: u64 = 0xfcfb6d1ba7725c30;
const FINAL_MAGIC: u64 = 0xf4fa6f45970d41d8;

const HEADER_SIZE: usize = 24;
const EOF_SIZE: usize = 24;
const SHA256_SIZE: usize = 32;

#[derive(Debug)]
struct CacheHeader {
    magic: u64,
    version: u32,
    key_length: u32,
    key_hash: u32,
}

#[derive(Debug)]
struct CacheEof {
    magic: u64,
    flags: u32,
    crc32: u32,
    stream_size: u32,
}

#[derive(Debug)]
struct CacheEntry {
    header: CacheHeader,
    key: Vec<u8>,

    stream1: Vec<u8>,
    stream1_eof: CacheEof,

    stream0: Vec<u8>,
    stream0_eof: CacheEof,

    key_sha256: Option<Vec<u8>>,
}

impl CacheEntry {
    /*
     * "Simple Cache" file contains:
     *
     * Header
     * Key
     * Stream 1
     * Stream 1 EOF
     * Stream 0
     * optional SHA256
     * Stream 0 EOF
     */
    fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() < HEADER_SIZE + EOF_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Cache file is too small",
            ));
        }

        let header = CacheHeader {
            magic: read_u64(data, 0)?,
            version: read_u32(data, 8)?,
            key_length: read_u32(data, 12)?,
            key_hash: read_u32(data, 16)?,
        };

        if header.magic != INITIAL_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid Simple Cache magic: {:#x}", header.magic),
            ));
        }

        let key_start = HEADER_SIZE;

        let key_end = key_start
            .checked_add(header.key_length as usize)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Key length overflow"))?;

        if key_end > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Key extends beyond file",
            ));
        }

        let key = data[key_start..key_end].to_vec();

        //The last EOF belongs to Stream 0.

        let final_eof_offset = data.len() - EOF_SIZE;

        let stream0_eof = parse_eof(data, final_eof_offset)?;

        if stream0_eof.magic != FINAL_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid final EOF magic: {:#x}", stream0_eof.magic),
            ));
        }

        //FLAG_HAS_KEY_SHA256 = bit 1.

        let has_sha256 = (stream0_eof.flags & 2) != 0;

        let sha_start = if has_sha256 {
            final_eof_offset.checked_sub(SHA256_SIZE).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid SHA256 position")
            })?
        } else {
            final_eof_offset
        };

        let key_sha256 = if has_sha256 {
            Some(data[sha_start..final_eof_offset].to_vec())
        } else {
            None
        };

        //END OF STREAM 0 and then
        //optional SHA256
        //EOF OF STREAM 0

        let stream0_end = sha_start;

        let stream0_size = stream0_eof.stream_size as usize;

        if stream0_size > stream0_end {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Stream 0 size",
            ));
        }

        let stream0_start = stream0_end - stream0_size;

        //Just before STREAM 0 is EOF OF STREAM 1
        if stream0_start < EOF_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing Stream 1 EOF?",
            ));
        }

        let stream1_eof_offset = stream0_start - EOF_SIZE;

        let stream1_eof = parse_eof(data, stream1_eof_offset)?;

        if stream1_eof.magic != FINAL_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid EOF magic of stream 1\t{:#x}", stream1_eof.magic),
            ));
        }

        //STREAM 1 begins after the key
        let stream1_start = key_end;
        let stream1_end = stream1_eof_offset;

        if stream1_end < stream1_start {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Stream 1 boundaries",
            ));
        }

        let stream1 = data[stream1_start..stream1_end].to_vec();

        //STREAM 0
        let stream0 = data[stream0_start..stream0_end].to_vec();

        Ok(Self {
            header,
            key,
            stream1,
            stream1_eof,
            stream0,
            stream0_eof,
            key_sha256,
        })
    }
}

fn parse_eof(data: &[u8], offset: usize) -> io::Result<CacheEof> {
    if offset + EOF_SIZE > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "EOF record extends beyond file",
        ));
    }

    Ok(CacheEof {
        magic: read_u64(data, offset)?,
        flags: read_u32(data, offset + 8)?,
        crc32: read_u32(data, offset + 12)?,
        stream_size: read_u32(data, offset + 16)?,
    })
}

fn read_u32(data: &[u8], offset: usize) -> io::Result<u32> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of file"))?;

    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn read_u64(data: &[u8], offset: usize) -> io::Result<u64> {
    let bytes = data
        .get(offset..offset + 8)
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected end of file"))?;

    Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
}

//Extract printable ASCII strings among the garbage
fn printable_strings(data: &[u8]) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = Vec::new();

    for &byte in data {
        if byte.is_ascii_graphic() || byte == b' ' || byte == b'\t' {
            current.push(byte);
        } else {
            if current.len() >= 4 {
                if let Ok(s) = String::from_utf8(current.clone()) {
                    result.push(s);
                }
            }

            current.clear();
        }
    }

    if current.len() >= 4 {
        if let Ok(s) = String::from_utf8(current) {
            result.push(s);
        }
    }

    result
}

//Find ALL HTTP and HTTPS URLs inside those bytes
fn find_urls(data: &[u8]) -> Vec<String> {
    let mut urls = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let start;

        if data[i..].starts_with(b"https://") {
            start = i;
            i += 8;
        } else if data[i..].starts_with(b"http://") {
            start = i;
            i += 7;
        } else {
            i += 1;
            continue;
        }

        while i < data.len() {
            let c = data[i];

            const VALID_CHARS: &[u8] =
                b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_./:?=&%#~@+!$'()*,;[]";

            let valid = VALID_CHARS.contains(&c);

            if !valid {
                break;
            }

            i += 1;
        }

        if let Ok(url) = std::str::from_utf8(&data[start..i]) {
            if !url.is_empty() {
                let decoded = match decode_proxy_url(url) {
                    Some(decoded) => decoded,
                    None => url.to_string(),
                };

                urls.push(decoded);
            }
        }
    }

    urls.sort_unstable();
    urls.dedup();
    urls
}

fn check_filetype(data: &[u8]) -> String {
    match infer::get(data) {
        Some(kind) => kind.extension().to_string(),
        None => "Unknown".to_string(),
    }
}

fn extract_content_type(stream0: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stream0);

    let lower = text.to_ascii_lowercase();

    let marker = "content-type:";

    if let Some(pos) = lower.find(marker) {
        let start = pos + marker.len();

        let value = text[start..].lines().next().unwrap_or("").trim();

        if !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}

fn decode_proxy_url(url: &str) -> Option<String> {
    let marker = "?u=";

    let pos = url.find(marker)?;

    let encoded = &url[pos + marker.len()..];

    let encoded = encoded.split('&').next().unwrap_or(encoded);

    urlencoding::decode(encoded).ok().map(|s| s.into_owned())
}

/*
fn write_recovered_file(
    cache_path: &Path,
    stream_number: usize,
    body: &[u8],
    kind: String,
) -> io::Result<PathBuf> {
    let parent = cache_path
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let stem = cache_path
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("cache-entry");

    let extension = kind
        .map(|(kind, _)| extension(kind))
        .unwrap_or("bin");

    let output = parent.join(format!(
        "{}.stream{}.recovered.{}",
        stem,
        stream_number,
        extension
    ));

    fs::write(&output, body)?;

    Ok(output)
}
*/
fn print_hex_preview(data: &[u8]) {
    const MAX: usize = 64;

    print!("    ");

    for byte in data.iter().take(MAX) {
        print!("{:02x} ", byte);
    }

    if data.len() > MAX {
        print!("...");
    }

    println!();
}

fn print_stream_strings(stream_name: &str, data: &[u8]) {
    let strings = printable_strings(data);

    if strings.is_empty() {
        return;
    }

    println!("Printable strings from {}:", stream_name);

    for string in strings.iter().take(30) {
        println!("    {}", string);
    }

    if strings.len() > 30 {
        println!("{} more strings (possibly garbage)", strings.len() - 30);
    }
}

fn print_data_information(stream_name: &str, data: &[u8]) {
    println!();
    println!("{} info:", stream_name);

    println!("Size: {}b", data.len());

    println!("Type: {}", check_filetype(data));
    //println!("  MIME: {}", check_filetype(data));

    println!();
    println!("Signature:");

    print_hex_preview(data);

    //print_stream_urls(stream_name, data);

    print_stream_strings(stream_name, data);
}

pub fn parse_entry(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let main_entry = CacheEntry::parse(&data);
    if let Ok(entry) = main_entry {
        //decode_proxy_url()
        Ok(find_urls(&entry.key))
    } else {
        Err("FUCK!".into())
    }
}

/*
fn parse_datacache(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(path)?;

    let main_entry = CacheEntry::parse(&data);

    //NORMAL ENTRY: Stream 0 + Stream 1
    if let Ok(entry) = main_entry {
        println!("Version:\t {}", entry.header.version);

        println!("Key length:\t{} bytes", entry.header.key_length);

        println!("Key hash:\t{:#010x}", entry.header.key_hash);

        println!("Cache key:");

        match String::from_utf8(entry.key.clone()) {
            Ok(key) => {
                println!("\t{}", key);
            }

            Err(_) => {
                println!("\t<binary key>");
                print_hex_preview(&entry.key);
            }
        }

        //URLs from cache key
        println!();
        println!("Request URLs:");

        let urls = find_urls(&entry.key);

        if urls.is_empty() {
            println!("\t<none>");
        } else {
            for url in &urls {
                println!("\t{}", url);

                if let Some(source) = decoded_proxy_url(url) {
                    if source != *url {
                        println!("\tSource URL:\t{}", source);
                    }
                }
            }
        }

        // STREAM 0
        println!("Stream 0");

        println!("Size: {}b", entry.stream0.len());

        if let Some(content_type) = extract_content_type(&entry.stream0) {
            println!("Content-Type:\t{}", content_type);
        } else {
            println!("Content-Type:\t<no idea>");
        }

        println!("Stream 0 signature:");

        print_hex_preview(&entry.stream0);

        print_stream_urls("Stream 0", &entry.stream0);

        print_stream_strings("Stream 0", &entry.stream0);

        // STREAM 1
        println!("Stream 1");

        print_data_information("Stream 1", &entry.stream1);

        let detected = check_filetype(&entry.stream1);

        //let output = write_recovered_file(path, 1, &entry.stream1, detected)?;

        return Ok(());
    } else {
        panic!("FUCKING SHIT");
    }
}
*/

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let argument = env::args().nth(1).ok_or("you just stupid")?;

    let path = PathBuf::from(argument);

    let a = parse_entry(&path)?;
    println!("{:?}", a);
    Ok(())
}

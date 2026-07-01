/*
 * Last 4 bytes of the every firefox's cache file is the byte number where the metadata starts.
 * There is NO DOCUMENTATION about this in firefox source, wow, thanks mozzila
 * IT HAS BEEN OVER 2 WEEKS, IT IS ALMOST AN ALPHA RELEASE
 * AND THE WHOLE PARSER STILL HASN'T BEEN FIXED TO NOT OUTPUT THE GARBAGE AT THE START!!!!!!!
 * I COMMAND YOU TO FIND A WAY TO CLEAN IT!!
 *
 * Nah, i think i will release Alpha without fixing this :) it's not like it's dangerous or anything anyway
 */

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

fn read_metadata(path: &str) -> io::Result<Vec<u8>> {
    //String> {
    let mut file = File::open(path)?;
    let file_size = file.seek(SeekFrom::End(0))?;
    //Get metadata beggining byte number
    file.seek(SeekFrom::End(-4))?;
    let mut buffer = [0u8; 4];
    file.read_exact(&mut buffer)?;
    let metadata_start = i32::from_be_bytes(buffer) as u64;
    let metadata_len = file_size - metadata_start - 4;
    file.seek(SeekFrom::Start(metadata_start))?;

    let mut metadata_raw = vec![0u8; metadata_len as usize];
    file.read_exact(&mut metadata_raw)?;

    //let metadata_str = String::from_utf8_lossy(&metadata_raw).into_owned();

    Ok(metadata_raw)
}

fn find_byte_sequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn parse_metadata(raw_metadata: &[u8]) -> Option<&str> {
    let needle = b"partitionKey=";

    let pos = find_byte_sequence(raw_metadata, needle)?;
    let start_pos = pos + needle.len();

    let colon_pos = raw_metadata[start_pos..].iter().position(|&b| b == b':')?;

    let url_start = start_pos + colon_pos + 1;

    let url_end = raw_metadata[url_start..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| url_start + p)
        .unwrap_or(raw_metadata.len());

    let url_bytes = &raw_metadata[url_start..url_end];

    let url = std::str::from_utf8(url_bytes).ok()?;

    Some(url)
}

/*
use std::env;
fn main() -> io::Result<()> {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => panic!("NO PATH PROVIDED"),
    };
        parse_metadata(read_metadata(&path)?);
    Ok(())
}
*/

pub fn get_metadata(path: &str) -> io::Result<String> {
    let raw_metadata = read_metadata(path)?;
    let result = parse_metadata(&raw_metadata).unwrap_or("").to_string();
    Ok(result)
}

use crate::browsette;
use crate::browsette::*;
use crate::cache2_entry_metadata;
use crate::dataset;
use crate::phash_generator;
use std::sync::mpsc::Sender;

use dirs::home_dir;
use ffmpeg_sidecar::command::FfmpegCommand;
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{env, fs};

use crate::gui_communication::*;

//Constants and statics, mainly paths. LazyLock is a saviour <3
//MOVE ALL THOSE TO MATCH FUNCTIONS

static BASE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = PathBuf::from("./");
    if !path.is_dir() {
        panic!("Cannot read base dir!: {}", path.display());
    }

    path
});

fn browser_history_scan(browser: &Browser, search_vector: &Vec<String>, tx: &Sender<GuiMessage>) {
    tx.send(GuiMessage::Log(LogMessage {
        message: format!("Scanning {}...", &browser.name),
        level: LogLevel::Info,
    }))
    .ok();
    let home_dir = home_dir().expect("No $HOME dir");

    let browser_config_profile_root = home_dir.join(browser.config_path);
    //get history file of a profile
    let profile_list_vector = get_profile_list(&browser);
    let profile_history = browser.history_file;

    for folder in profile_list_vector {
        let folder = PathBuf::from(folder);
        //firefox and its forks uses places.sqlite, chrome uses History which is (sqlite3)
        let history_file = browser_config_profile_root
            .join(folder.as_path())
            .join(&profile_history);
        if history_file.is_file() {
            tx.send(GuiMessage::Log(LogMessage {
                message: format!("Scanning {}...", history_file.display()),
                level: LogLevel::Info,
            }))
            .ok();
            let conn = Connection::open(history_file).expect("Cannot open history database");

            let query = match browser.family {
                BrowserFamily::Gecko => "SELECT url, title FROM moz_places WHERE url LIKE ?1",
                BrowserFamily::Chromium => "SELECT url, title FROM urls WHERE url LIKE ?1",
            };

            match conn.prepare(query) {
                Ok(mut response) => {
                    for search in search_vector {
                        //build search querry
                        let pattern = format!("%{}%", search);

                        //prepare query that will search for stuff and execute it
                        let mut rows = response.query(params![pattern]).expect("Query failed");

                        // loop through rows
                        while let Some(row) = rows.next().expect("Failed to fetch row") {
                            let url: Option<String> = row.get(0).unwrap_or_default();
                            let title: Option<String> = row.get(1).unwrap_or_default();
                            tx.send(GuiMessage::Log(LogMessage {
                                message: format!(
                                    "Found: url: {}, title:{}!",
                                    url.unwrap_or_default(),
                                    title.unwrap_or_default()
                                ),
                                level: LogLevel::Info,
                            }))
                            .ok();
                        }
                    }
                }
                Err(error) => {
                    if let rusqlite::Error::SqliteFailure(err, _) = error {
                        match err.code {
                            rusqlite::ErrorCode::DatabaseBusy => {
                                tx.send(GuiMessage::Log(
                                            LogMessage{
                                                message: "The browser history database is locked, perhaps the browser is still running? Close it first. No attempt to scan."
                                                .into(),
                                                level: LogLevel::Warning,
                                            }
                                        )
                                ).ok();
                            }
                            _ => {
                                tx.send(GuiMessage::Log(
                                            LogMessage{
                                                message: format!(
                                                    "Failed to prepare query due to an error: {:#?}. No attempt to scan.",
                                                    error
                                                ),
                                                level: LogLevel::Error,
                                            }
                                        )
                                ).ok();
                            }
                        }
                    } else {
                        tx.send(GuiMessage::Log(
                                    LogMessage{
                                        message: format!(
                                            "Failed to prepare query due to an error: {:#?}. No attempt to scan.",
                                            error
                                        ),
                                        level: LogLevel::Error
                                    }
                                )
                        )
                        .ok();
                    }
                }
            };
        } else {
            tx.send(GuiMessage::Log(LogMessage {
                message: format!(
                    "{} does not exists. No attempt to scan",
                    history_file.display(),
                ),
                level: LogLevel::Warning,
            }))
            .ok();
        }
    }
}

// check file mime type
fn check_filetype(path: impl AsRef<Path>) -> String {
    //Option<String> {
    match infer::get_from_path(path).expect("Fuk") {
        Some(kind) => kind.extension().to_string(),
        None => "Unknown".to_string(),
    }
}

// Copying from and to (Un)Veryfied dir
fn safely_copy(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> std::io::Result<()> {
    let (source, destination) = (source.as_ref(), destination.as_ref());
    if !destination.is_file() {
        fs::copy(source, destination)?;
    } else {
        println!("{} already in {}", source.display(), destination.display());
    }
    Ok(())
}
//Check if mp4 file is a complete file or just a part of a longer video
fn check_if_video_stream_is_complete() {
    todo!();
}

fn browser_cache_asset_scan(browser: &Browser, asset_data: &[String], tx: &Sender<GuiMessage>) {
    println!(
        "Scanning {}'s cache for asset_data.txt entries...",
        browser.name
    );

    let home_dir = home_dir().expect("Cannot read $HOME");

    let profile_list_vector = get_profile_list(&browser);

    let browser_cache_profile_root = home_dir.join(browser.cache_path);
    let profile_cache = browser.cache_entries_path;

    for folder in profile_list_vector {
        let folder_cache_path = &browser_cache_profile_root.join(folder).join(&profile_cache);

        if folder_cache_path.is_dir() {
            print!("Scanning {:?}", folder_cache_path);
            if let Ok(cache_entries) = fs::read_dir(&folder_cache_path) {
                for cache_entry in cache_entries {
                    let cache_entry_path = cache_entry.unwrap().path();
                    //is it a file
                    if cache_entry_path.is_file() {
                        //initialize vector of difference values
                        let cache_entry_file_name = &cache_entry_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .into_owned();
                        //println!("Checking {}", cache_entry_file_name);

                        let entry_url =
                            cache2_entry_metadata::get_metadata(cache_entry_path.to_str().unwrap())
                                .expect("Unknown problem reading entry's metadata");

                        //println!("{:?}", entry_url);

                        for (i, asset_data_entry) in asset_data.iter().enumerate() {
                            print!("{i} /{}\r", asset_data.len());

                            if entry_url.contains(asset_data_entry) {
                                println!("Found");
                            }
                        }
                        println!();

                        /*
                            for (i, video_data_entry) in video_data.into_iter().enumerate() {
                                for file in fs::read_dir(&potential_file_path).unwrap() {
                                    let path = file.unwrap().path();
                                    let filepath = path.to_str().unwrap();
                                    let hash_to_check = phash_generator::generate_phash(filepath);
                                    for &video_entry_hash in &video_data_entry.hash {
                                        let result = hamming(video_entry_hash, hash_to_check);
                                        similarity.push(result);
                                    }
                                    match similarity.iter().min().cloned() {
                                        //Some(v) => println!(
                                        //	"Closest similarity to {} is {}",
                                        //	&cache_entry_path.to_str().unwrap(),
                                        //	v
                                        //),
                                        Some(v) => similarity_pack.push(v),
                                        None => println!("No hashes in vector!"),
                                    }
                                }
                                use std::io::{self, Write};
                                print!("{i} /600\r");
                                io::stdout().flush().unwrap();
                                let similarity_final = similarity_pack.iter().min().unwrap();
                                //only if similarity difference is less than 5
                                if *similarity_final < 5 as u32 {
                                    println!();
                                    println!("Closest similarity of {:?} is {:?}", video_data_entry.title, similarity_final);
                                    let copy_destination = PathBuf::from("./Unverified/{}").join(&cache_entry_path);
                                    safely_copy(&cache_entry_path, PathBuf::from(copy_destination)).expect("Poop");
                                }
                            }

                            //remove temp directories with raw files afterwards
                            if potential_file_path.is_dir() {
                                fs::remove_dir_all(&potential_file_path).unwrap();
                            }
                        }*/
                    }
                }
            } else {
                println!("Cannot read folder {:?}", folder_cache_path);
            }
        } else {
            println!("No cache folder found in profile {:?}", folder_cache_path)
        }
    }
}

/// Scans browser's cache for video files
fn browser_cache_video_scan(
    browser: &Browser,
    video_data: &[dataset::VideoData],
    tx: &Sender<GuiMessage>,
) {
    tx.send(GuiMessage::Log(LogMessage {
        message: format!(
            "Scanning {}'s cache for video_data.txt entries...",
            browser.name
        ),
        level: LogLevel::Info,
    }))
    .ok();

    let home_dir = home_dir().expect("Cannot read $HOME");

    let profile_list_vector = get_profile_list(&browser);

    let browser_cache_profile_root = home_dir.join(browser.cache_path);
    let profile_cache = browser.cache_entries_path;

    for folder in profile_list_vector {
        let folder_cache_path = &browser_cache_profile_root.join(folder).join(&profile_cache);

        if folder_cache_path.is_dir() {
            tx.send(GuiMessage::Log(LogMessage {
                message: format!("Scanning {:?}", folder_cache_path),
                level: LogLevel::Info,
            }))
            .ok();

            if let Ok(cache_entries) = fs::read_dir(&folder_cache_path) {
                for cache_entry in cache_entries {
                    let cache_entry_path = cache_entry.unwrap().path();
                    //is it a file and a video file
                    if cache_entry_path.is_file() {
                        //initialize vector of difference values

                        let filetype = check_filetype(&cache_entry_path);

                        if ["mp4", "webm", "flv"].contains(&filetype.as_str()) {
                            //|| infer::is_image(&buf){
                            //println!("{:?}", cache_entry_path);
                            //extract frame and gen hash
                            let cache_entry_file_name = &cache_entry_path
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .into_owned();

                            tx.send(GuiMessage::Log(LogMessage {
                                message: format!("Checking {}", cache_entry_file_name),
                                level: LogLevel::Info,
                            }))
                            .ok();

                            //if the temporary dir is there remove it
                            //if not, create it, use it and then remove it
                            let potential_file_path = env::temp_dir().join(cache_entry_file_name);
                            if potential_file_path.is_dir() {
                                fs::remove_dir_all(&potential_file_path).unwrap();
                            }
                            fs::create_dir(&potential_file_path).unwrap();
                            let tmp_file = env::temp_dir()
                                .join(&cache_entry_file_name)
                                .join("frame_%03d.raw");

                            extract_videoframes(
                                PathBuf::from(&cache_entry_path),
                                PathBuf::from(&tmp_file),
                            );

                            for (i, video_data_entry) in video_data.into_iter().enumerate() {
                                let mut difference = Vec::new();
                                let mut difference_pack = Vec::new();
                                for file in fs::read_dir(&potential_file_path).unwrap() {
                                    let path = file.unwrap().path();
                                    let filepath = path.to_str().unwrap();
                                    let hash_to_check = phash_generator::generate_phash(filepath);
                                    for &video_entry_hash in &video_data_entry.hash {
                                        let result = phash_generator::hamming(
                                            video_entry_hash,
                                            hash_to_check,
                                        );
                                        difference.push(result);
                                    }
                                    match difference.iter().min().cloned() {
                                        //Some(v) => println!(
                                        //	"Closest similarity to {} is {}",
                                        //	&cache_entry_path.to_str().unwrap(),
                                        //	v
                                        //),
                                        Some(v) => difference_pack.push(v),
                                        None => println!("No hashes in vector!"),
                                    }
                                }

                                //                                use std::io::{self, Write};
                                //                                print!("{i} /{}\r", video_data.len());
                                //                                io::stdout().flush().unwrap();

                                tx.send(GuiMessage::Progress(ProgressMessage {
                                    progress: (i + 1) as f32 / video_data.len() as f32,
                                    progress_total: (i + 1) as f32 / video_data.len() as f32,
                                }))
                                .ok();

                                let difference_final = difference_pack.iter().min().unwrap();
                                // only if difference is less than 5
                                if *difference_final < 5 as u32 {
                                    tx.send(GuiMessage::Log(LogMessage {
                                        message: format!(
                                            "Closest difference of {:?} is {:?}",
                                            video_data_entry.title, difference_final
                                        ),
                                        level: LogLevel::Info,
                                    }))
                                    .ok();

                                    let copy_destination =
                                        PathBuf::from("./Verified/{}").join(&cache_entry_path);
                                    safely_copy(&cache_entry_path, PathBuf::from(copy_destination))
                                        .expect("Couldn't!");
                                }
                            }

                            //remove temp directories with raw files afterwards
                            if potential_file_path.is_dir() {
                                fs::remove_dir_all(&potential_file_path).unwrap();
                            }
                        }
                    }
                }
            } else {
                tx.send(GuiMessage::Log(LogMessage {
                    message: format!("Cannot read folder {:?}", folder_cache_path),
                    level: LogLevel::Error,
                }))
                .ok();
            }
        } else {
            tx.send(GuiMessage::Log(LogMessage {
                message: format!("No cache folder found in profile {:?}", folder_cache_path),
                level: LogLevel::Error,
            }))
            .ok();
        }
    }
}

fn extract_videoframes(input_file: PathBuf, output_file: PathBuf) {
    //extracts first frame of video into grayscale 32x32 raw
    let input = input_file.to_str().expect("Invalid input path");
    let output = output_file.to_str().expect("Invalid output path");

    FfmpegCommand::new()
        .args([
            "-i",
            input,
            "-vf",
            "scale=32:32",
            "-pix_fmt",
            "gray",
            "-f",
            //"rawvideo",
            "image2",
            output,
        ])
        .spawn()
        .expect("You need ffmpeg installed for scanning cached videos!")
        .wait()
        .expect("ffmpeg gave up :(");
}

// Do it all
pub fn process(tx: Sender<GuiMessage>) {
    tx.send(GuiMessage::Log(LogMessage {
        message: "Starting...".into(),
        level: LogLevel::Info,
    }))
    .ok();
    //load browser paths
    let linux_browser_paths = browsette::SUPPORTED_BROWSERS;

    //detect browsers installed on the pc
    let detected_browsers = detect_browsers(linux_browser_paths);

    tx.send(GuiMessage::Log(LogMessage {
        message: "Detected browsers:".into(),
        level: LogLevel::Info,
    }))
    .ok();
    for browser in &detected_browsers {
        tx.send(GuiMessage::Log(LogMessage {
            message: format!("\t{} at {}", browser.name, browser.config_path),
            level: LogLevel::Info,
        }))
        .ok();
    }

    //load dataset
    let dataset = dataset::load_dataset(BASE_DIR.join("data")); //<-- DONE

    tx.send(GuiMessage::Log(
                LogMessage {
                    message: format!(
                                "Loaded database:\n\tvideo_data: {},\n\twatch_page_data: {},\n\tasset_data: {},\n\thistory_data: {}",
                                dataset.video_data.len(),
                                dataset.watch_page_data.len(),
                                dataset.asset_data.len(),
                                dataset.history_data.len()
                    ),
                    level: LogLevel::Info
                }
        )
    ).ok();

    for browser in &detected_browsers {
        //search video ids in browser history
        browser_history_scan(&browser, &dataset.history_data, &tx); //<--DONE FOR LIBREWOLF/FIREFOX/CHROME/CHROMIUM
    }
    for browser in &detected_browsers {
        browser_cache_video_scan(&browser, &dataset.video_data, &tx); //<--DONE FOR LIBREWOLF/FIREFOX/CHROME/CHROMIUM
    }

    /*
    for browser in &detected_browsers {
        browser_cache_asset_scan(&browser, &dataset.asset_data); //TODO
    }*/
    tx.send(GuiMessage::Log(LogMessage {
        message: "Done!".into(),
        level: LogLevel::Info,
    }))
    .ok();
}

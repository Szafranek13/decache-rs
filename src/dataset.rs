use std::fs::read_to_string;
use std::path::{Path, PathBuf};
///This struct is meant consume contents of files in "data" directory
///See VideoData for special treatment of video_data.txt
pub struct DataSet {
    pub video_data: Vec<VideoData>,
    pub watch_page_data: Vec<String>,
    pub asset_data: Vec<String>,
    pub history_data: Vec<String>,
}

///This struct is meant to consume contents of video_data.txt.
///Every line is split by "|" separator and every part is put to the corresponding key
///
///Example:
///
///"Roblox+Video+101|-8128339497863628282,39c9cb3870db6751|0000000000000000|00:03:52.43|00:03:52.63"
///
///VIDEO TITLE|GOOGLE_VIDEO_ID,GOOGLE_VIDEO_CONTENT_ID|HASH|DUR_MIN|DUR_MAX
#[derive(Debug)]
pub struct VideoData {
    //IN video_data.txt THIS IS A REGEX SPERATED BY + (i think)
    ///Title of a video.
    ///It's meant to consume a string with + signs instead of spaces. Perhaps a regex attempt.
    pub title: String,
    ///Ids of a video.
    ///It's meant to consume vector of strings out of a string splited by ",".
    pub ids: Vec<String>,
    ///Hash of a video.
    ///It's meant to consume a u64 converted from a string.
    pub hash: Vec<u64>,
    ///Minimal duration of a searched video.
    pub duration_min: String,
    ///Maximal duration of a searched video.
    pub duration_max: String,
}

fn read_lines(path: impl AsRef<Path>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let read_string = read_to_string(path.as_ref())?
        .lines()
        .map(String::from)
        .collect();
    Ok(read_string)
}

///Loads all files from "data" directory into fields of DataSet.
pub fn load_dataset(data_dir: PathBuf) -> Result<DataSet, Box<dyn std::error::Error>> {
    //Loading dataset
    //Read lines of video_data.txt
    let video_data_path = data_dir.join("video_data.txt");
    let video_data = read_lines(&video_data_path)?;

    let mut video_data_struct_vec: Vec<VideoData> = Vec::new();

    //process each line of the file
    for entry in video_data {
        //remove quotes from the line
        let entry_sanitised = &entry
            .strip_prefix('"')
            .and_then(|entry| entry.strip_suffix('"'))
            .unwrap_or(&entry);
        //split the line by "|" into a vector
        let entry_vec: Vec<&str> = entry_sanitised.split('|').collect();

        if entry_vec.len() != 5 {
            // panic when not enough data!!!

            return Err(format!(
                "Expected 5 fields, got {} in line: {}",
                entry_vec.len(),
                entry_sanitised
            )
            .into());
        }
        //create VideoData structure from vector
        let entry_struct = VideoData {
            title: entry_vec[0].to_string(),
            ids: entry_vec[1]
                .split(",") //split ids by ","
                .map(|p| p.to_string())
                .collect(),
            hash: entry_vec[2]
                .split(",")
                .filter_map(|h| Some(u64::from_str_radix(h, 16).unwrap_or(0)))
                .collect(),
            duration_min: entry_vec[3].to_string(),
            duration_max: entry_vec[4].to_string(),
        };
        video_data_struct_vec.push(entry_struct);
    }

    /*println!(
        "Loaded {} entries from {}",
        video_data_struct_vec.len(),
        video_data_path.display()
    );*/
    let watch_page_data_path = data_dir.join("watch_page_data.txt");
    let watch_page_data = read_lines(&watch_page_data_path)?;
    /*println!(
        "Loaded {} entries from {}",
        watch_page_data.len(),
        watch_page_data_path.display()
    );*/
    let asset_data_path = data_dir.join("asset_data.txt");
    let asset_data = read_lines(&asset_data_path)?;
    /*println!(
        "Loaded {} entries from {}",
        asset_data.len(),
        asset_data_path.display()
    );*/
    let history_data_path = data_dir.join("history_data.txt");
    let history_data = read_lines(&history_data_path)?;
    /*println!(
        "Loaded {} entries from {}",
        history_data.len(),
        history_data_path.display()
    );*/

    Ok(DataSet {
        video_data: video_data_struct_vec,
        watch_page_data,
        asset_data,
        history_data,
    })
}

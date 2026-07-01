// The name of this crate made by me is a reference to Bowser turning into a woman meme :D
use dirs::home_dir;
use ini::Ini;
use serde_json::Value;
use std::fmt;
use std::fs;

#[derive(Debug)]
pub struct Browser {
    pub name: BrowserName,
    pub family: BrowserFamily,
    pub config_path: &'static str,
    pub profiles_file: &'static str,
    pub history_file: &'static str,
    pub cache_path: &'static str,
    pub cache_index_file: &'static str,
    pub cache_entries_path: &'static str,
}

#[derive(Debug, Clone)]
pub enum BrowserName {
    Firefox,
    LibreWolf,
    Chrome,
    Chromium,
}

impl fmt::Display for BrowserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Firefox => write!(f, "Firefox"),
            Self::LibreWolf => write!(f, "Librewolf"),
            Self::Chrome => write!(f, "Chrome"),
            Self::Chromium => write!(f, "Chromium"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BrowserFamily {
    Gecko,
    Chromium,
}

// CHANGE THIS CODE BELLOW. For example a linux user might want to scan their Window's' drive
#[cfg(target_os = "linux")]
pub static SUPPORTED_BROWSERS: &[Browser] = LINUX_SUPPORTED_BROWSERS;
#[cfg(target_os = "windows")]
pub static SUPPORTED_BROWSERS: &[Browser] = WINDOWS_SUPPORTED_BROWSERS;

/// Slice of structs with common data of said browsers

/// Slice of structs with data about all covered browsers
pub static LINUX_SUPPORTED_BROWSERS: &[Browser] = &[
    Browser {
        name: BrowserName::Firefox,
        family: BrowserFamily::Gecko,
        config_path: ".mozilla/firefox",
        profiles_file: "profiles.ini",
        history_file: "places.sqlite",
        cache_path: ".cache/mozilla/firefox",
        cache_index_file: "cache2/index",
        cache_entries_path: "cache2/entries",
    },
    Browser {
        name: BrowserName::LibreWolf,
        family: BrowserFamily::Gecko,
        config_path: ".config/librewolf/librewolf",
        profiles_file: "profiles.ini",
        history_file: "places.sqlite",
        cache_path: ".cache/librewolf",
        cache_index_file: "cache2/index",
        cache_entries_path: "cache2/entries",
    },
    Browser {
        name: BrowserName::Chrome,
        family: BrowserFamily::Chromium,
        config_path: ".config/google-chrome",
        profiles_file: "Local State",
        history_file: "History",
        cache_path: ".cache/google-chrome",
        cache_index_file: "Cache/index-dir/the-real-index",
        cache_entries_path: "Cache/Cache_Data", //apparently, the older one used different one?
    },
    Browser {
        name: BrowserName::Chromium,
        family: BrowserFamily::Chromium,
        config_path: ".config/chromium",
        profiles_file: "Local State",
        history_file: "History",
        cache_path: ".cache/chromium",
        cache_index_file: "Cache/index-dir/the-real-index",
        cache_entries_path: "Cache/Cache_Data",
    },
];

pub static WINDOWS_SUPPORTED_BROWSERS: &[Browser] = &[
    Browser {
        name: BrowserName::Firefox,
        family: BrowserFamily::Gecko,
        config_path: "AppData\\Roaming\\Mozilla\\Firefox",
        profiles_file: "profiles.ini",
        history_file: "places.sqlite",
        cache_path: "AppData\\Local\\Mozilla\\Firefox",
        cache_index_file: "cache2\\index",
        cache_entries_path: "cache2\\entries",
    },
    Browser {
        name: BrowserName::LibreWolf,
        family: BrowserFamily::Gecko,
        config_path: "AppData\\Roaming\\librewolf",
        profiles_file: "profiles.ini",
        history_file: "places.sqlite",
        cache_path: "AppData\\Local\\librewolf",
        cache_index_file: "cache2/index",
        cache_entries_path: "cache2/entries",
    },
    Browser {
        name: BrowserName::Chrome,
        family: BrowserFamily::Chromium,
        config_path: "AppData\\Local\\Google\\Chrome\\User Data",
        profiles_file: "Local State",
        history_file: "History",
        cache_path: "AppData\\Local\\Google\\Chrome\\User Data",
        cache_index_file: "Cache/index-dir/the-real-index",
        cache_entries_path: "Cache/Cache_Data", //apparently, the older one used different one?
    },
    Browser {
        name: BrowserName::Chromium,
        family: BrowserFamily::Chromium,
        config_path: "AppData\\Local\\Chromium\\User Data",
        profiles_file: "Local State",
        history_file: "History",
        cache_path: "AppData\\Local\\Chromium\\User Data",
        cache_index_file: "Cache/index-dir/the-real-index",
        cache_entries_path: "Cache/Cache_Data",
    },
];

pub fn detect_browsers(browser_paths: &[Browser]) -> Vec<&Browser> {
    #[cfg(target_os = "linux")]
    let home_dir = home_dir().expect("Cannot read $HOME");
    #[cfg(target_os = "windows")]
    let home_dir = home_dir().expect("Cannot read %USERPROFILE%");

    let mut detected_browser_paths = Vec::new();
    for browser in browser_paths {
        if home_dir.join(browser.config_path).is_dir() & home_dir.join(browser.cache_path).is_dir()
        {
            detected_browser_paths.push(browser);
        }
    }
    detected_browser_paths
}

//#[cfg(target_os = "linux")]
pub fn get_profile_list(browser: &Browser) -> Vec<String> {
    let home_dir = home_dir().expect("Cannot read $HOME");

    let browser_config_profile_root = home_dir.join(browser.config_path);
    let profile_list_file_path = browser_config_profile_root.join(browser.profiles_file);
    let mut profile_list_vector: Vec<String> = Vec::new();

    let profiles_list_file_content =
        fs::read_to_string(profile_list_file_path).expect("Could not read file.");

    match browser.family {
        BrowserFamily::Gecko => {
            let profiles_list_ini = Ini::load_from_str(&profiles_list_file_content).unwrap();
            for (section, props) in profiles_list_ini.iter() {
                if let Some(section_name) = section {
                    if section_name.starts_with("Profile") {
                        match props.get("Path") {
                            Some(path) => profile_list_vector.push(path.to_owned()),
                            None => panic!("Profile section is missing Path value, skipping..."),
                        }
                    }
                }
            }
        }
        BrowserFamily::Chromium => {
            let profile_list_json: Value =
                serde_json::from_str(&profiles_list_file_content).unwrap();

            if let Some(profiles) = profile_list_json["profile"]["info_cache"].as_object() {
                for (profile_dir, _) in profiles {
                    profile_list_vector.push(profile_dir.to_owned())
                }
            } else {
                panic!("No profiles found in Local State");
            }
        }
    }

    profile_list_vector
}

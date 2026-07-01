# ![](logo.png) Decache-rs

A complete Rust rewrite of Decache by SindexMon https://github.com/SindexMon/decache/ (and all it's components) aimed to have GUI and work on Linux, Windows and Mac.
It is aimed to be fully compatibile with assets of the original Decache.

## pre-Alpha version (0.1.2) for Linux has been released!

> [!CAUTION]
> This is a pre-Alpha release, expect lots of bugs, not working stuff and false positives.

> [!NOTE]
> You need ffmpeg installed on your system for video_data entries to be scanned! For now the software will crash without it.

### How to use:

1. Download precompiled decache-rs from releases
2. Copy `data` directory from the original Decache
3. Put it in the same directory as decache-rs executable
4. Start the program
![](screenshot.png)
In pre-Alpha version Deache-rs will scan for entries from history_data.txt, video_data.txt and asset_data.txt in browsers' cache directories on your linux system (supports Firefox, LibreWolf, Chrome and Chromium, only Firefox and LibreWolf supports asset_data searching for now Chromium and Chrome gives wrong results about asset_data). All the scanning will be done in place (for now only found lost media of video_data.txt will be put into Verified folder (it will be created automaticaly when media will be found). It will display positive results (ex. "Found XYZ!") as green messages in the log view. Progress of each entry is shown on a progressbar. No found lost media will be sent to SindexMon or anywhere in this version.

### You can build it from source too:
Use `cargo build --release --target x86_64-unknown-linux-gnu` to build it for linux.

[![Developed by a human not by AI](https://notbyai.fyi/img/developed-by-human-not-by-ai-white.svg)](https://notbyai.fyi/)

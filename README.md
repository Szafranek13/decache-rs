# Decache port for Linux
## THIS SOFTWARE IS NOT YET READY TO BE USED BEWARE

A rewrite of Decache by SindexMon https://github.com/SindexMon/decache/ for Linux in rust.
It is aimed to be fully compatibile with assets of original decache.

### What is done:
1. Scanning for entries from `history_data.txt` in browser history of every profile of Firefox, Librewolf, Chrome and Chromium
2. Scanning browser cache of Firefox and Librewolf and comparing hashes of found videos' frames with hashes of entries from `video_data.txt`

Use `cargo build --release --target x86_64-unknown-linux-gnu` to build it

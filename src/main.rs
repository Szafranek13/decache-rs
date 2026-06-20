//No AI was used to write this program, I have alergy to ai slop.
//Also, i learned rust like 1 year ago, so this probably can be optimised
//
//28.05.2026 Added parsing data/video_data.txt into struct
//29.05.2026 Added parsing data/watch_page_data.txt, data/asset_data.txt, data/history_data.txt
//   into vectors, added searching through firefox's browsing history, added filetype checker,
//   added primitive browser cache scanner
//30.05.2026 Path are now proper PathBuf type not &str. Added FFmpeg extractor of frames from
//   videos. Added pHash generator from images. Added primitive comparator of hashes.
//31.05.2026 Replaced the good, well functioning pHash generator with ai generated translation of
//   pHash.cpp because they gave different results, improved frame extractor to use image2 instead of rawvideo,
//   improved comparator of hashes
//01.06.2026 Made "hash" of VideoData structure a vector.
//   Hashing works well, and fast

// TODO Do something about the ffmpeg bottlneck maybe...
// TODO The most important functions (or all of them) should return Result propperly instead of panicing
// TODO Chrome/Chromium stores cache in a weird format, process it
// TODO Original skips looking into cache entries that are from web.archive.org
// TODO LogMessage should be a vector containing the separate messages. Currently LogMessage is a
// really long string containing all of messages Strings. Egui can't color that.

//The original script seems to copy only MP4 FLV and WEBM video files to Unveryfied
//It also checks if a video file it found is complete by checking if it has ftyp at the beggining of file
//if it doesnt then it's not a first piece of a video, but the middle or the final, and then it
//concentate them
//

mod browsette;
mod cache2_entry_metadata;
mod cache2_metadata;
mod dataset;
mod phash_generator;
mod scanner;

use crate::scanner::process;
use eframe::egui;
use egui::Ui;

mod gui_communication;
use crate::gui_communication::*;

use std::sync::mpsc::{self, Receiver, Sender};

struct MyApp {
    log: Vec<LogMessage>,
    progress: f32,
    progress_total: f32,
    rx: Receiver<GuiMessage>,
    tx: Sender<GuiMessage>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            log: vec![LogMessage {
                message: "Press Start to start!\n".to_string(),
                level: LogLevel::Info,
            }],
            progress: 0.0,
            progress_total: 0.0,
            rx,
            tx,
        }
    }
}

pub fn main() -> eframe::Result {
    //    egui_logger::builder().init().unwrap();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Decache-rs debug",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::default()))),
    )
}

impl eframe::App for MyApp {
    fn ui(&mut self, _: &mut Ui, _: &mut eframe::Frame) {}
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        while let Ok(output) = self.rx.try_recv() {
            match output {
                GuiMessage::Log(log) => {
                    self.log.push(log);
                }

                GuiMessage::Progress(progress) => {
                    self.progress = progress.progress as f32;
                    self.progress_total = progress.progress_total as f32;
                }
            }
            //self.log.push(LogMessage{message:'\n'.to_string(),level:LogLevel::Info});
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Decache-rs");

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.set_height(ui.available_height()); //VERY WRONG THING TO DO
                ui.set_width(ui.available_width());
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in &self.log {
                            match entry.level {
                                LogLevel::Info => {
                                    ui.label(&entry.message);
                                }

                                LogLevel::Warning => {
                                    ui.colored_label(egui::Color32::YELLOW, &entry.message);
                                }

                                LogLevel::Error => {
                                    ui.colored_label(egui::Color32::RED, &entry.message);
                                }
                            }
                        }
                    });
            });
        });

        egui::Panel::bottom("controls").show(ctx, |ui| {
            ui.add(
                egui::widgets::ProgressBar::new(self.progress)
                    .fill(egui::Color32::DARK_BLUE)
                    .show_percentage(),
            );

            ui.add(
                egui::widgets::ProgressBar::new(self.progress_total)
                    .fill(egui::Color32::DARK_GREEN)
                    .show_percentage(),
            );

            ui.horizontal(|ui| {
                if ui
                    .add_sized([50.0, 25.0], egui::Button::new("Quit"))
                    .clicked()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_sized([50.0, 25.0], egui::Button::new("Start"))
                        .clicked()
                    {
                        let tx = self.tx.clone();
                        std::thread::spawn(move || {
                            process(tx);
                        });
                    }
                });
            });
        });
    }
}

//No AI was used to write this program, I have alergy to ai slop.
//Also, i learned rust like 1 year ago, so this probably can be optimised
//

// TODO Do something about the ffmpeg bottlneck maybe...
// TODO The most important functions (or all of them) should return Result propperly instead of panicing
// TODO Chrome/Chromium stores cache in a weird format, process it
// TODO Original skips looking into cache entries that are from web.archive.org

//The original script seems to copy only MP4 FLV and WEBM video files to Unveryfied
//It also checks if a video file it found is complete by checking if it has ftyp at the beggining of file
//if it doesnt then it's not a first piece of a video, but the middle or the final, and then it
//concentate them
//

// I wanted to release alpha much earlier but pride month was more important sorry guys :(

mod browsette;
mod cache2_entry_metadata;
mod dataset;
mod phash_generator;
mod scanner;

use crate::scanner::process;
use eframe::egui;

mod gui_communication;
use crate::gui_communication::*;

use std::sync::mpsc::{self, Receiver, Sender};

struct MyApp {
    log: Vec<LogMessage>,
    progress: f32,
    progress_total: f32,
    rx: Receiver<GuiMessage>,
    tx: Sender<GuiMessage>,
    processing: bool,
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
            processing: false,
        }
    }
}

pub fn main() -> eframe::Result {
    //    egui_logger::builder().init().unwrap();
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../logo.png")).unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        &format!("Decache-rs {}", env!("DECACHE_VERSION")),
        options,
        Box::new(|cc| Ok(Box::new(MyApp::default()))),
    )
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.request_repaint();
        while let Ok(output) = self.rx.try_recv() {
            match output {
                GuiMessage::Log(log) => {
                    self.log.push(log);
                }

                GuiMessage::Progress(progress) => {
                    self.progress = progress.progress as f32;
                    self.progress_total = progress.progress_total as f32;
                }

                GuiMessage::Finished => {
                    self.processing = false;
                }
            }
            //self.log.push(LogMessage{message:'\n'.to_string(),level:LogLevel::Info});
        }
        egui::CentralPanel::default().show_inside(ui, |ui| {
            //main label
            ui.horizontal(|ui| {
                //                ui.add(
                //                    egui::Image::new(egui::include_image!("../logo.png"))
                //                        .fit_to_exact_size(egui::vec2(32.0, 32.0))
                //                );
                ui.heading("Decache-rs");
                ui.label(egui::RichText::new(env!("DECACHE_VERSION")).color(egui::Color32::ORANGE));
                ui.label(egui::RichText::new("built"));
                ui.label(egui::RichText::new(env!("BUILD_DATE")).color(egui::Color32::YELLOW));
                ui.label(egui::RichText::new("for"));
                ui.label(egui::RichText::new(env!("BUILD_TARGET")).color(egui::Color32::CYAN));
            });

            egui::Panel::bottom("controls").show_inside(ui, |ui| {
                ui.add(
                    egui::widgets::ProgressBar::new(self.progress)
                        .fill(egui::Color32::DARK_BLUE)
                        .show_percentage(),
                );

                // ui.add(
                //     egui::widgets::ProgressBar::new(self.progress_total)
                //         .fill(egui::Color32::DARK_GREEN)
                //         .show_percentage(),
                // );

                ui.horizontal(|ui| {
                    if ui
                        .add_sized([50.0, 25.0], egui::Button::new("Quit"))
                        .clicked()
                    {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !self.processing,
                                egui::Button::new("Start").min_size(egui::vec2(50.0, 25.0)),
                            )
                            .clicked()
                        {
                            self.processing = true;

                            let tx = self.tx.clone();

                            std::thread::spawn(move || {
                                process(tx);
                            });
                        }
                    });
                });
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.set_height(ui.available_height()); //VERY WRONG THING TO DO
                ui.set_width(ui.available_width());
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
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

                                LogLevel::Good => {
                                    ui.colored_label(egui::Color32::GREEN, &entry.message);
                                }
                            }
                        }
                    });
            });
        });
    }
}

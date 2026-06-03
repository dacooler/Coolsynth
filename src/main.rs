#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use std::{collections::HashMap, sync::{Arc, Mutex}};

use eframe::{egui, wgpu::naga::compact::KeepUnused::No};
use egui::{Key, ScrollArea};
mod sound;
use rodio::{MixerDeviceSink, Player, mixer::Mixer};
use sound::play_note;

use crate::sound::Envelope;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    eframe::run_native(
        "Keyboard events",
        options,
        Box::new(|_cc| {
            Ok(Box::new(Content {
                text: "".to_string(),
                handle: handle,
                envelopes: vec![None, None, None, None, None, None, None],
            }))
        }),
    )
}

struct Content {
    text: String,
    handle: MixerDeviceSink,
    envelopes: Vec<Option<Arc<Mutex<Envelope>>>>,
}

impl eframe::App for Content {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Press/Hold/Release example. Press A to test.");
            if ui.button("Clear").clicked() {
                self.text.clear();
            }
            ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(&self.text);
                });
            let keys = vec![
                ("A", 440.0),
                ("S", 493.88),
                ("D", 523.25),
                ("F", 587.33),
                ("G", 659.25),
                ("H", 698.46),
                ("J", 783.99),
            ];
            for (index, (key, freq)) in keys.iter().enumerate(){
                if ui.input(|i| i.key_pressed(Key::from_name(key).unwrap())) {
                    if self.envelopes[index].is_none() {
                        self.envelopes[index] = Some(play_note(&mut self.handle, *freq));
                    }
                }
                if ui.input(|i| i.key_released(Key::from_name(key).unwrap())) && let Some(envelope) = &self.envelopes[index] {
                    envelope.lock().unwrap().sustained = false;
                    self.envelopes[index] = None;
                }
            };
        });
    }
}

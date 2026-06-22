#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex}};

use eframe::{egui, wgpu::naga::compact::KeepUnused::No};
use egui::{Key, ScrollArea};
mod sound;
use rodio::{MixerDeviceSink, Player, mixer::Mixer};
use sound::{play_note, MasterAudio};

use crate::sound::{AudioGenerator, Envelope, SynthValues};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    let values = Arc::new(Mutex::new(vec![ 10000.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]));
    let toggles = Arc::new(Mutex::new(vec![false, false, false]));
    let master_audio = MasterAudio::new(values.clone(), toggles.clone()); 
    let sources = master_audio.sources.clone();
    
    handle.mixer().add(master_audio);
    eframe::run_native(
        "Keyboard events",
        options,
        Box::new(|_cc| {
            Ok(Box::new(Content {
                text: "".to_string(),
                values: values.clone(),
                toggles: toggles.clone(),
                sources: sources,
                envelopes: vec![None, None, None, None, None, None, None],
            }))
        }),
    )
}

struct Content {
    text: String,
    sources: Arc<Mutex<Vec<AudioGenerator>>>,
    envelopes: Vec<Option<Arc<Mutex<Envelope>>>>,
    values: Arc<Mutex<Vec<f32>>>,
    toggles: Arc<Mutex<Vec<bool>>>,
}
static SYNTH_VALUES: LazyLock<SynthValues> = LazyLock::new(|| SynthValues::new(0.0, 0.0));

impl eframe::App for Content {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Press/Hold/Release example. Press A to test.");
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[0], 1.0..=10000.0).text("cutoff"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[2], 1.0..=10000.0).text("cutoff env"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[1], 1.0..=10.0).text("resonance"));

            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[3], 0.001..=10.0).text("cutoff A"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[4], 0.001..=10.0).text("cutoff D"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[5], 0.001..=1.0).text("cutoff S"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[6], 0.001..=10.0).text("cutoff R"));

            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[7], 0.001..=10.0).text("volume A"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[8], 0.001..=10.0).text("volume D"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[9], 0.001..=1.0).text("volume S"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[10],0.001..=10.0).text("volume R"));

            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[11],0.0..=1.0).text("Distortion"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[12],0.0..=1.0).text("Chorus"));
            ui.add(egui::Slider::new(&mut self.values.lock().unwrap()[13],0.0..=1.0).text("Delay"));
            ui.add(egui::Checkbox::new(&mut self.toggles.lock().unwrap()[0], "chorus"));
            ui.add(egui::Checkbox::new(&mut self.toggles.lock().unwrap()[1], "delay"));
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
                        self.envelopes[index] = Some(play_note(self.sources.clone(), *freq, self.values.clone()));
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

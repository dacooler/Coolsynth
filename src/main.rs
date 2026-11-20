#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{self, Write, stdin, stdout},
    sync::{Arc, LazyLock, Mutex},
};

use indexmap::IndexMap;

use eframe::{egui, wgpu::naga::compact::KeepUnused::No};
use egui::{Key, ScrollArea};
mod sound;
use midir::{Ignore, MidiInput};
use rodio::{MixerDeviceSink, Player, mixer::Mixer};
use serde_json::Value;
use sound::{MasterAudio, MasterAudioIterator, play_note, stop_playing};
use std::thread;

use crate::sound::{AudioGenerator, Envelope, SynthValues};

struct Sounds {
    pub envelopes: Arc<Mutex<IndexMap<u8, Arc<Mutex<Envelope>>>>>,
    pub sources: Arc<Mutex<Vec<AudioGenerator>>>,
    pub values: Arc<Mutex<Vec<f32>>>,
}

fn main() -> eframe::Result {
    let values = Arc::new(Mutex::new(vec![
        10000.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]));
    let toggles = Arc::new(Mutex::new(vec![false, false, false]));
    let sources = Arc::new(Mutex::new(Vec::new()));
    let envelopes = Arc::new(Mutex::new(IndexMap::new()));
    let master_audio = MasterAudio::new(
        values.clone(),
        toggles.clone(),
        sources.clone(),
        envelopes.clone(),
    );

    let sounds = Sounds {
        envelopes: envelopes.clone(),
        sources: sources.clone(),
        values: values.clone(),
    };

    let options = eframe::NativeOptions::default();
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");

    handle.mixer().add(MasterAudioIterator {
        master_audio: master_audio,
    });

    thread::spawn(move || match run(sounds) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    });

    eframe::run_native(
        "Keyboard events",
        options,
        Box::new(|_cc| {
            Ok(Box::new(Content {
                text: "".to_string(),
                values: values.clone(),
                toggles: toggles.clone(),
                id_counter: 1,
            }))
        }),
    )
}

struct Content {
    text: String,
    values: Arc<Mutex<Vec<f32>>>,
    toggles: Arc<Mutex<Vec<bool>>>,
    id_counter: i32,
}
static SYNTH_VALUES: LazyLock<SynthValues> = LazyLock::new(|| SynthValues::new(0.0, 0.0));

impl eframe::App for Content {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Press/Hold/Release example. Press A to test.");
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[0], 1.0..=10000.0)
                    .text("cutoff"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[2], 1.0..=10000.0)
                    .text("cutoff env"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[1], 1.0..=10.0)
                    .text("resonance"),
            );

            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[3], 0.001..=10.0)
                    .text("cutoff A"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[4], 0.001..=10.0)
                    .text("cutoff D"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[5], 0.001..=1.0)
                    .text("cutoff S"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[6], 0.001..=10.0)
                    .text("cutoff R"),
            );

            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[7], 0.001..=10.0)
                    .text("volume A"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[8], 0.001..=10.0)
                    .text("volume D"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[9], 0.001..=1.0)
                    .text("volume S"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[10], 0.001..=10.0)
                    .text("volume R"),
            );

            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[11], 0.0..=1.0)
                    .text("Distortion"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[12], 0.0..=1.0).text("Chorus"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[13], 0.0..=1.0).text("Delay"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[14], 0.0..=5.0)
                    .text("Delay time"),
            );
            ui.add(
                egui::Slider::new(&mut self.values.lock().unwrap()[15], 0.0..=1.0)
                    .text("Delay feedback"),
            );
            ui.add(egui::Checkbox::new(
                &mut self.toggles.lock().unwrap()[0],
                "chorus",
            ));
            ui.add(egui::Checkbox::new(
                &mut self.toggles.lock().unwrap()[1],
                "delay",
            ));
            if ui.button("save").clicked() {
                let mut values: Vec<f32> = vec![];
                for value in self.values.clone().lock().unwrap().iter_mut() {
                    values.push(*value);
                }
                let _ = write_json(values);
            }
            if ui.button("load").clicked() {
                let _ = load_json(self.values.clone());
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
                ("K", 880.0),
                ("L", 987.76),
            ];
            /*
            for (index, (key, freq)) in keys.iter().enumerate() {
                if ui.input(|i| i.key_pressed(Key::from_name(key).unwrap())) {
                    self.id_counter++;
                    play_note(*freq, self.values.clone(), self.id_counter);
                }
                if ui.input(|i| i.key_released(Key::from_name(key).unwrap()))
                    && let Some(envelope) = &self.envelopes[index]
                {
                    envelope.lock().unwrap().sustained = false;
                    self.envelopes[index] = None;
                }
            }
            */
        });
    }
}

fn run(sounds: Sounds) -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            println!("{}: {:?} (len = {})", stamp, message, message.len());
            println!("{:?}", message[1]);
            if (message[2] == 127) {
                play_note(
                    sounds.envelopes.clone(),
                    sounds.sources.clone(),
                    sounds.values.clone(),
                    message[1],
                );
            }
            if (message[2] == 0) {
                stop_playing(sounds.envelopes.clone(), message[1]);
            }
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}

fn write_json(data: Vec<f32>) -> Result<(), io::Error> {
    let json = serde_json::to_string(&data)?;
    let path = "saved.json";
    let mut output = File::create(path)?;
    write!(output, "{}", json)?;

    Ok(())
}

fn load_json(values: Arc<Mutex<Vec<f32>>>) -> Result<(), io::Error> {
    let json = fs::read_to_string("saved.json").expect("file should be able to be read");
    let data: Vec<f32> = serde_json::from_str(&json)?;
    println!("{:?}", data);
    for i in 0..=data.len() - 1 {
        values.lock().unwrap()[i] = data[i];
    }
    Ok(())
}

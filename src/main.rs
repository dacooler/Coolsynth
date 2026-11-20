#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write, stdin, stdout},
    sync::{Arc, LazyLock},
};

use arc_swap::ArcSwap;

use eframe::egui;
use egui::ScrollArea;
mod sound;
use midir::{Ignore, MidiInput};
use sound::MasterAudio;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use crate::sound::SynthValues;

fn main() -> eframe::Result {
    let values = Arc::new(ArcSwap::from_pointee(vec![
        10000.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]));
    let toggles = Arc::new(ArcSwap::from_pointee(vec![false, false, false]));

    let (send, recv): (Sender<(u8, u8, u8)>, Receiver<(u8, u8, u8)>) = channel();

    let master_audio = MasterAudio::new(values.clone(), toggles.clone(), recv);

    let options = eframe::NativeOptions::default();
    let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");

    handle.mixer().add(master_audio);

    thread::spawn(move || match run(send) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    });

    eframe::run_native(
        "Keyboard events",
        options,
        Box::new(|_cc| {
            Ok(Box::new(Content {
                text: "".to_string(),
                values,
                toggles,
            }))
        }),
    )
}

struct Content {
    text: String,
    values: Arc<ArcSwap<Vec<f32>>>,
    toggles: Arc<ArcSwap<Vec<bool>>>,
}
static SYNTH_VALUES: LazyLock<SynthValues> = LazyLock::new(|| SynthValues::new(0.0, 0.0));

impl eframe::App for Content {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            let mut new_values = (**self.values.load()).clone();
            ui.heading("Press/Hold/Release example. Press A to test.");
            ui.add(egui::Slider::new(&mut new_values[0], 1.0..=10000.0).text("cutoff"));
            ui.add(egui::Slider::new(&mut new_values[1], 1.0..=10.0).text("resonance"));
            ui.add(egui::Slider::new(&mut new_values[2], 1.0..=10000.0).text("cutoff env"));
            ui.add(egui::Slider::new(&mut new_values[3], 0.001..=10.0).text("cutoff A"));
            ui.add(egui::Slider::new(&mut new_values[4], 0.001..=10.0).text("cutoff D"));
            ui.add(egui::Slider::new(&mut new_values[5], 0.001..=1.0).text("cutoff S"));
            ui.add(egui::Slider::new(&mut new_values[6], 0.001..=10.0).text("cutoff R"));
            ui.add(egui::Slider::new(&mut new_values[7], 0.001..=10.0).text("volume A"));
            ui.add(egui::Slider::new(&mut new_values[8], 0.001..=10.0).text("volume D"));
            ui.add(egui::Slider::new(&mut new_values[9], 0.001..=1.0).text("volume S"));
            ui.add(egui::Slider::new(&mut new_values[10], 0.001..=10.0).text("volume R"));
            ui.add(egui::Slider::new(&mut new_values[11], 0.0..=1.0).text("Distortion"));
            ui.add(egui::Slider::new(&mut new_values[12], 0.0..=1.0).text("Chorus"));
            ui.add(egui::Slider::new(&mut new_values[13], 0.0..=1.0).text("Delay"));
            ui.add(egui::Slider::new(&mut new_values[14], 0.0..=5.0).text("Delay time"));
            ui.add(egui::Slider::new(&mut new_values[15], 0.0..=1.0).text("Delay feedback"));

            if new_values != **self.values.load() {
                self.values.store(Arc::new(new_values));
            }

            let mut new_toggles = (**self.toggles.load()).clone();
            ui.add(egui::Checkbox::new(&mut new_toggles[0], "chorus"));
            ui.add(egui::Checkbox::new(&mut new_toggles[1], "delay"));

            if new_toggles != **self.toggles.load() {
                self.toggles.store(Arc::new(new_toggles));
            }

            if ui.button("save").clicked() {
                let mut values: Vec<f32> = vec![];
                for value in self.values.load().iter() {
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

fn run(sender: Sender<(u8, u8, u8)>) -> Result<(), Box<dyn Error>> {
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
            match sender.send((message[1], message[2], message[0])) {
                Err(_error) => {}
                Ok(_result) => {}
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

fn load_json(values: Arc<ArcSwap<Vec<f32>>>) -> Result<(), io::Error> {
    let json = fs::read_to_string("saved.json")?;
    let data: Vec<f32> = serde_json::from_str(&json)?;
    println!("{:?}", data);
    values.store(Arc::new(data));
    Ok(())
}

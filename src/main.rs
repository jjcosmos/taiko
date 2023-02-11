use core::fmt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::{stdout, Read, Write};
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use eframe::egui;
use eframe::egui::Id;
use json_structures::custom::Config;
use json_structures::edda_objects::Root;

use crate::converters::MidiConverter;

mod converters;
mod json_structures;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Takes a midi file and outputs the conversion to the given path.
    Convert { source: String, output_file: String },
    /// Takes a midi file and outputs multiple data files based off of track names.
    Auto {
        source: String,
        output_folder: String,
    },
    /// Lets the user configure taiko
    Configure,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        run_cli()
    }
    else {
        run_gui()
    }
}

fn run_gui() {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(egui::vec2(570.0, 300.0)),
        ..Default::default()
    };

    let title = format!("{} v{}", NAME, VERSION);
    eframe::run_native(
        title.as_str(),
        options,
        Box::new(|_cc| {
            Box::new(MyApp {
                config: get_or_create_config(),
                log: vec!["\t".to_string(), "\t".to_string(), "\t".to_string()],
                ..Default::default()
            })
        }),
    );
}

#[derive(Default)]
struct MyApp {
    source_path: Option<String>,
    output_path: Option<String>,
    output_type: ComboBoxConversion,
    difficulty: Difficulty,
    config: Config,
    log: Vec<String>,
}

#[derive(PartialEq, Debug)]
enum ComboBoxConversion {
    SingleOutput,
    MultiOutput,
}

impl fmt::Display for ComboBoxConversion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleOutput => write!(f, "Single"),
            Self::MultiOutput => write!(f, "Multi"),
        }
    }
}

#[derive(PartialEq, Debug)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Easy => write!(f, "Easy"),
            Self::Normal => write!(f, "Normal"),
            Self::Hard => write!(f, "Hard"),
        }
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        Difficulty::Easy
    }
}

impl Default for ComboBoxConversion {
    fn default() -> Self {
        ComboBoxConversion::SingleOutput
    }
}

impl MyApp {
    fn log_str(&mut self, msg: String) {
        println!("{}", msg);
        self.log.push(msg);
        if self.log.len() > 3 {
            self.log.remove(0);
        }
    }

    fn write_output_app(&mut self, path: &String, data: &Root) {
        let file_name = path;
        match File::create(file_name) {
            Ok(mut file) => {
                println!("Writing to {} ...", file_name);
                match file.write_all(
                    serde_json::to_string_pretty(data)
                        .unwrap_or_default()
                        .as_bytes(),
                ) {
                    Ok(_) => self.log_str(format!("Success! Wrote to: {}", file_name)),
                    Err(e) => self.log_str(format!("Failed to create file {}: {}", path, e)),
                }
            }
            Err(e) => self.log_str(format!("Failed to create file {}: {}", path, e)),
        }
    }

    fn save_config_app(&mut self, config: &Config, file: &mut File) {
        match serde_json::to_string_pretty(config) {
            Ok(json_str) => {
                match file.set_len(0) {
                    Ok(_) => {},
                    Err(e) => self.log_str(format!("Failed to save config: {}", e)),
                };
                match file.write_all(json_str.as_bytes()) {
                    Ok(_) => self.log_str(format!("Saved config!")),
                    Err(e) => self.log_str(format!("Failed to save config: {}", e)),
                };
                    
            }
            Err(e) => {
                self.log_str(format!("{}", e));
            }
        };
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right(Id::new("right frame")).default_width(80_f32).show(ctx, |ui|{
            for (i, midi) in self.config.drum_map.iter_mut().enumerate() {
                ui.with_layout(egui::Layout::top_down_justified(eframe::emath::Align::Min), |ui| {
                    ui.label(format!("Drum {}:", i));
                    ui.add(egui::DragValue::new(midi));
                    *midi = u8::clamp(*midi, 0, 127);
                });
            }

            ui.with_layout(egui::Layout::top_down_justified(eframe::emath::Align::Min), |ui| {
                if ui.button("Save Config").clicked() {
                    match get_or_create_file_rw(&Path::new("config.json")) {
                        Ok(mut file) => {
                            self.save_config_app(&self.config.clone(), &mut file);
                        }
                        Err(e) => {
                            self.log_str(format!("Could not save config: {}", e));
                        }
                    };
                }
            });  
        });

        egui::TopBottomPanel::bottom(Id::new("console")).show(ctx, |ui| {
            let layout = egui::Layout::top_down(eframe::emath::Align::Min).with_cross_justify(false);
            ui.with_layout(layout , |ui|{
                if !self.log.is_empty() {
                    ui.label("Log:");
                        for log_line in &self.log {
                            ui.label(log_line);
                        }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ComboBox::from_label("File Output Type")
                .selected_text(format!("{}", self.output_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.output_type,
                        ComboBoxConversion::SingleOutput,
                        "Single",
                    );
                    ui.selectable_value(
                        &mut self.output_type,
                        ComboBoxConversion::MultiOutput,
                        "Multi",
                    );
                });

            if self.output_type == ComboBoxConversion::SingleOutput {
                egui::ComboBox::from_label("Difficulty")
                    .selected_text(format!("{:?}", self.difficulty))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.difficulty, Difficulty::Easy, "Easy");
                        ui.selectable_value(&mut self.difficulty, Difficulty::Normal, "Normal");
                        ui.selectable_value(&mut self.difficulty, Difficulty::Hard, "Hard");
                    });
            }

            ui.horizontal(|ui| {
                if ui.button("Select Source").clicked() {
                    let filter = ["midi", "mid", "MIDI", "MID"];
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Midi Files", &filter)
                        .pick_file()
                    {
                        self.source_path = Some(path.display().to_string());
                    }
                }
            });

            if let Some(source) = &self.source_path {
                let def = source.as_str();
                let split = source.split(std::path::MAIN_SEPARATOR).collect::<Vec<&str>>();
                let name = split.last().unwrap_or(&def);

                ui.horizontal(|ui| {
                    ui.label("Source:");
                    ui.monospace(name.to_string());
                });
            }

            ui.horizontal(|ui| {
                if ui.button("Select Destination").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.output_path = Some(path.display().to_string());
                    }
                }
            });

            if let Some(picked_path) = &self.output_path {
                let path = match self.output_type {
                    ComboBoxConversion::SingleOutput => {
                        format!("{}{}{:?}.dat", &picked_path, std::path::MAIN_SEPARATOR, self.difficulty)
                    }
                    ComboBoxConversion::MultiOutput => format!(
                        "{}{}{:?}, {:?}, {:?}.dat",
                        &picked_path,
                        std::path::MAIN_SEPARATOR,
                        Difficulty::Easy,
                        Difficulty::Normal,
                        Difficulty::Hard
                    ),
                };

                let def = path.as_str();
                let split = picked_path.split(std::path::MAIN_SEPARATOR).collect::<Vec<&str>>();
                let output_name = split.last().unwrap_or(&def);

                ui.horizontal(|ui| {
                    ui.label("Output:");
                    ui.monospace(output_name.to_string());
                });
            }

            if let (Some(source), Some(output)) =
                (&self.source_path.clone(), &self.output_path.clone())
            {
                if ui.button("Run").clicked() {
                    self.log.clear();
                    match self.output_type {
                        ComboBoxConversion::SingleOutput => {
                            match MidiConverter::new(source.to_string(), &self.config)
                                .to_root_merge_notes_and_meta()
                            {
                                Ok(r) => {
                                    let mut path_buf = PathBuf::new();
                                    path_buf.push(output);
                                    path_buf.push(format!("{}{}", self.difficulty, &self.config.batch_output_extension));
                                    self.write_output_app(&path_buf
                                        .into_os_string()
                                        .into_string()
                                        .unwrap_or_default(), &r);
                                },
                                Err(e) => self.log_str(format!("Error: {}", e)),
                            };
                        }
                        ComboBoxConversion::MultiOutput => {
                            match MidiConverter::new(source.to_string(), &self.config)
                                .to_root_merge_meta()
                            {
                                Ok(r) => {
                                    for res in r {
                                        let mut path_buf = PathBuf::new();
                                        path_buf.push(output.clone());
                                        path_buf.push(format!(
                                            "{}{}",
                                            &res.0, &self.config.batch_output_extension
                                        ));
                                        self.write_output_app(
                                            &path_buf
                                                .into_os_string()
                                                .into_string()
                                                .unwrap_or_default(),
                                            &res.1,
                                        );
                                    }
                                }
                                Err(e) => self.log_str(format!("Error: {}", e)),
                            }
                        }
                    };
                }
            }
        });
    }
}

fn run_cli() {
    let config = get_or_create_config(); // This should be failable. No point in parsing without a config.
    let args = Args::parse();
    handle_cli_input(args, config);
}

fn handle_cli_input(args: Args, config: Config) {
    match args.command {
        Commands::Convert {
            source,
            output_file,
        } => match MidiConverter::new(source, &config).to_root_merge_notes_and_meta() {
            Ok(r) => write_output(&output_file, &r),
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Auto {
            source,
            output_folder,
        } => match MidiConverter::new(source, &config).to_root_merge_meta() {
            Ok(r) => {
                for res in r {
                    let mut path_buf = PathBuf::new();
                    path_buf.push(output_folder.clone());
                    path_buf.push(format!("{}{}", &res.0, &config.batch_output_extension));
                    write_output(
                        &path_buf.into_os_string().into_string().unwrap_or_default(),
                        &res.1,
                    );
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Configure => {
            println!("Midi Pitch Config:");

            let mut map = config.drum_map;
            let mut batch_extension = config.batch_output_extension.clone();

            for (i, map_val) in map.iter_mut().enumerate() {
                let mut buf = String::new();
                loop {
                    print!(
                        "Position [{}] is currently set to <{}>. Please enter the new value: ",
                        i, map_val
                    );
                    stdout().flush().unwrap();
                    match io::stdin().read_line(&mut buf) {
                        Ok(_) => match buf.trim().parse::<u8>() {
                            Ok(parsed) => {
                                *map_val = parsed;
                                break;
                            }
                            Err(e) => {
                                eprintln!("{}", e)
                            }
                        },
                        Err(e) => {
                            eprintln!("{}", e)
                        }
                    };
                }
            }

            println!("Output Extention:");

            loop {
                print!(
                    "Current batch extension is <{}>. Please enter the new value: ",
                    batch_extension
                );
                stdout().flush().unwrap();
                let mut buf = String::new();
                match io::stdin().read_line(&mut buf) {
                    Ok(_) => {
                        batch_extension = buf.trim().to_string();
                        break;
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }

            let new_config = Config {
                drum_map: map,
                batch_output_extension: batch_extension,
            };

            match get_or_create_file_rw(&Path::new("config.json")) {
                Ok(mut file) => {
                    save_config(&new_config, &mut file);
                }
                Err(e) => {
                    eprintln!("Could not save config: {}", e);
                }
            };
        }
    }
}

fn save_config(config: &Config, file: &mut File) {
    match serde_json::to_string_pretty(config) {
        Ok(json_str) => {
            file.set_len(0).expect("Failed to write config.");
            file.write_all(json_str.as_bytes())
                .expect("Failed to write config.");
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    };
}

fn get_or_create_file_rw(path: &Path) -> Result<File, std::io::Error> {
    if !path.exists() {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
    } else {
        OpenOptions::new().read(true).write(true).open(path)
    }
}

fn get_or_create_config() -> Config {
    let path = Path::new("config.json");

    match get_or_create_file_rw(&path) {
        Ok(mut file) => {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).unwrap_or_default();
            match serde_json::from_str(buffer.as_str()) {
                Ok(config) => config,
                Err(_) => {
                    save_config(&Config::default(), &mut file);
                    eprintln!("Failed to read config. Using default.");
                    return Config::default();
                }
            }
        }
        Err(_) => {
            eprintln!("Failed to read config. Using default.");
            return Config::default();
        }
    }
}

fn write_output(path: &String, data: &Root) {
    let file_name = path;
    match File::create(file_name) {
        Ok(mut file) => {
            println!("Writing to {} ...", file_name);
            file.write_all(
                serde_json::to_string_pretty(data)
                    .unwrap_or_default()
                    .as_bytes(),
            )
            .expect("Failed to serialize data");
        }
        Err(e) => eprintln!("Failed to create file {}: {}", path, e),
    }
}

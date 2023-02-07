use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::{stdout, Read, Write};
use std::path::Path;

use clap::Parser;
use clap::Subcommand;
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
    Convert {
        source: String,
        output_file: String,
    },
    /// Takes a midi file and outputs multiple data files based off of track names.
    Auto {
        source: String,
        output_folder: String,
    },
    /// Lets the user configure taiko
    Configure,
}

fn main() {
    let config = get_or_create_config(); // This should be failable. No point in parsing without a config.
    let args = Args::parse();

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
                    let path = output_folder.clone() + &res.0 + &config.batch_output_extension;
                    write_output(&path, &res.1);
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
        Err(e) => eprintln!("Failed to create file : {}", e),
    }
}

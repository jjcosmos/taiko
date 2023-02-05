use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use json_structures::custom::Config;
use json_structures::edda_objects::Root;

use crate::converters::MidiConverter;

mod converters;
mod json_structures;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source: String,
    #[arg(short, long)]
    dest: String,
}

fn main() {
    let config = get_or_create_config();
    let args = Args::parse();
    let converter: MidiConverter = MidiConverter::new(args.source, config);

    match converter.to_root() {
        Ok(r) => write_output(args.dest, &r),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn get_or_create_config() -> Config {
    let path = Path::new("config.json");
    let default_config = Config {
        drum_map: (60..63).collect(),
    };

    if !path.exists() {
        let mut f = File::create(path).expect("Could not create config file.");
        f.write_all(
            serde_json::to_string_pretty(&default_config)
                .unwrap_or_default()
                .as_bytes(),
        )
        .unwrap();
    }

    match File::open(path) {
        Ok(f) => {
            let reader = BufReader::new(f);
            return match serde_json::from_reader(reader) {
                Ok(config) => config,
                Err(_) => default_config,
            };
        }
        Err(e) => {
            eprintln!("{}", e);
            return default_config;
        }
    }
}

fn write_output(path: String, data: &Root) {
    let file_name = path;
    match File::create(file_name) {
        Ok(mut file) => {
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

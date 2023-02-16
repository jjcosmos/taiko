#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use clap::Parser;
use clap::Subcommand;
use taiko_app::get_or_create_config;
use taiko_app::TaikoApp;

mod converters;
mod json_structures;
mod taiko_app;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
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

fn main() {
    let config = get_or_create_config();
    let app = TaikoApp::from_config(config);

    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        app.run_cli()
    } else {
        app.run_gui()
    }
}

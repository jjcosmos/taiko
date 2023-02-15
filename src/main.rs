use clap::Parser;
use clap::Subcommand;
use eframe::egui;
use taiko_app::get_or_create_config;
use taiko_app::handle_cli_input;
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

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        run_cli()
    } else {
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
            Box::new(TaikoApp {
                config: get_or_create_config(),
                log: vec!["\t".to_string(), "\t".to_string(), "\t".to_string()],
                ..Default::default()
            })
        }),
    );
}

fn run_cli() {
    let config = get_or_create_config(); // This should be failable. No point in parsing without a config.
    let args = Args::parse();
    handle_cli_input(args, config);
}

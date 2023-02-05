use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use json_structures::custom::Config;
use midly::MetaMessage;

use serde_json::Value;

use crate::json_structures::edda_objects::*;

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
    let default_drum_map = vec![60, 61, 62, 63];
    let config = match get_or_create_config() {
        Some(c) => c,
        None => {
            println!(
                "config not found. using default of {}",
                default_drum_map
                    .clone()
                    .into_iter()
                    .map(|i| i.to_string() + " ")
                    .collect::<String>()
            );
            Config {
                drum_map: default_drum_map,
            }
        }
    };
    let args = Args::parse();
    println!("converting {} -> {}", args.source, args.dest);

    // If config is empty, could run through it twice to grab all used notes and just use midi order?
    // TODO: Take multiple midi tracks and parse the difficulty from the name. Sort of like a mini batch mode

    let input_file = args.source;
    let data = std::fs::read(input_file).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();

    if smf.tracks.len() < 1 {
        eprint!("No tracks!");
        return;
    }

    let track0 = &smf.tracks[0];
    let mut stamped_hits: Vec<Note> = vec![];
    let mut bpm_changes: Vec<Bpmchange> = vec![];
    let mut ticks_elapsed: u128 = 0;

    let mut tick_len: f64 = 0_f64;
    let mut current_beat_len: f64 = 0.0;
    let mut global_beat_accumulator: f64 = 0.0;

    for event in track0 {
        ticks_elapsed = ticks_elapsed + event.delta.as_int() as u128;
        let _time_accumulator = (ticks_elapsed as f64 * tick_len) / 1_000_000_f64;
        global_beat_accumulator += if current_beat_len != 0.0 {
            event.delta.as_int() as f64 / current_beat_len
        } else {
            0.0
        };

        match event.kind {
            midly::TrackEventKind::Midi {
                channel: _,
                message,
            } => match message {
                midly::MidiMessage::NoteOn { key, vel } => {
                    if vel != 0 {
                        stamped_hits.push(Note {
                            line_index: config
                                .drum_map
                                .iter()
                                .position(|&r| r == key.as_int())
                                .unwrap_or(0) as i64,
                            time: global_beat_accumulator,
                            line_layer: 1,
                            type_field: 0,
                            cut_direction: 1,
                        });
                    }
                }
                _ => {}
            },
            midly::TrackEventKind::SysEx(_) => {}
            midly::TrackEventKind::Escape(_) => {}
            midly::TrackEventKind::Meta(m) => match m {
                MetaMessage::Tempo(tempo) => {
                    let mut beat_len: f64 = 0.0;
                    tick_len = match smf.header.timing {
                        midly::Timing::Metrical(m) => {
                            current_beat_len = m.as_int() as f64;
                            let microseconds = tempo.as_int() as f64 / m.as_int() as f64;
                            beat_len = m.as_int() as f64 * microseconds;
                            microseconds.into()
                        }
                        midly::Timing::Timecode(_f, _u) => {
                            println!("Timecode mode not tested. Results may not be correct. Use metrical for best results.");
                            1.0 / _f.as_f32() as f64 / _u as f64
                        }
                    };
                    let bpm = 60_f64 / (beat_len / 1_000_000_f64);

                    if let Some(change) = bpm_changes.last_mut() {
                        if change.time == global_beat_accumulator {
                            change.bpm = bpm;
                        } else {
                            let mut copy = change.clone();
                            copy.bpm = bpm;
                            copy.time = global_beat_accumulator;
                            bpm_changes.push(copy);
                        }
                    } else {
                        let change = Bpmchange {
                            bpm: bpm,
                            time: global_beat_accumulator,
                            beats_per_bar: 4,
                            metronome_offset: 4,
                        };
                        bpm_changes.push(change);
                    }
                }
                MetaMessage::TimeSignature(numerator, _denominator, _clocks_per_click, _b) => {
                    if let Some(change) = bpm_changes.last_mut() {
                        if change.time == global_beat_accumulator {
                            change.beats_per_bar = numerator as i64;
                            change.metronome_offset = numerator as i64;
                            // I don't know what metranome offset is. Relevant? Idk.
                        } else {
                            let mut copy = change.clone();
                            copy.beats_per_bar = numerator as i64;
                            copy.metronome_offset = numerator as i64;
                            copy.time = global_beat_accumulator;
                            bpm_changes.push(copy);
                        }
                    } else {
                        let change = Bpmchange {
                            bpm: 120_f64,
                            time: global_beat_accumulator,
                            beats_per_bar: numerator as i64,
                            metronome_offset: numerator as i64,
                        };
                        bpm_changes.push(change);
                    }
                }
                _ => {}
            },
        }
    }

    let json_data = Root {
        version: "1".to_string(),
        custom_data: CustomData {
            time: 0,
            bpmchanges: bpm_changes,
            bookmarks: Vec::<Value>::new(),
        },
        events: Vec::<Value>::new(),
        notes: stamped_hits,
        obstacles: Vec::<Value>::new(),
    };

    write_output(args.dest, &json_data);
}

fn get_or_create_config() -> Option<Config> {
    let path = Path::new("config.json");

    if !path.exists() {
        let mut f = File::create(path).expect("Failed to create config.");
        let conf = Config {
            drum_map: vec![60, 61, 62, 63],
        };
        f.write_all(
            serde_json::to_string_pretty(&conf)
                .unwrap_or_default()
                .as_bytes(),
        )
        .expect("Failed to create default config");
    }

    match File::open(path) {
        Ok(f) => {
            let reader = BufReader::new(f);
            return match serde_json::from_reader(reader) {
                Ok(d) => Some(d),
                Err(_) => None,
            };
        }
        Err(e) => {
            eprintln!("{}", e);
            return None;
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

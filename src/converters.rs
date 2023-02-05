use midly::MetaMessage;
use serde_json::Value;

use crate::json_structures::{
    custom::Config,
    edda_objects::{Bpmchange, CustomData, Note, Root},
};

pub struct MidiConverter {
    source: String,
    configuration: Config,
}

impl MidiConverter {
    pub fn new(source: String, configuration: Config) -> Self {
        MidiConverter {
            source,
            configuration,
        }
    }

    pub fn to_root(&self) -> Result<Root, &'static str> {
        let input_file = &self.source;
        let data = std::fs::read(input_file).unwrap();
        let smf = midly::Smf::parse(&data).unwrap();

        if smf.tracks.len() < 1 {
            return Err("No tracks!");
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
                                line_index: self
                                    .configuration
                                    .drum_map
                                    .iter()
                                    .position(|&r| r == key.as_int())
                                    .unwrap_or(0)
                                    as i64,
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

        Ok(json_data)
    }
}

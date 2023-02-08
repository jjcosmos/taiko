use midly::{MetaMessage, Smf, TrackEvent, TrackEventKind};
use serde_json::Value;
use std::{str, vec};

use crate::json_structures::{
    custom::Config,
    edda_objects::{Bpmchange, CustomData, Note, Root},
};

#[derive(Copy, Clone)]
struct StampedEvent<'a> {
    event: TrackEvent<'a>,
    ticks_elapsed: u64,
}

struct OffsetEvent<'a> {
    event: TrackEvent<'a>,
    delta: u64,
}

struct TrackAsOffsets<'a> {
    offsets: Vec<OffsetEvent<'a>>,
}

impl TrackAsOffsets<'_> {
    fn track_name(&self) -> Option<String> {
        for offset in self.offsets.iter() {
            match offset.event.kind {
                TrackEventKind::Meta(m) => {
                    match m {
                        MetaMessage::TrackName(name) => {
                            return match str::from_utf8(name) {
                                Ok(name) => Some(name.to_string()),
                                Err(_) => None,
                            };
                        } ,
                        _ => {},
                    }
                },
                _ => {},
            }
        }

        return None;
    }
}

impl StampedEvent<'_> {
    fn get_meta_events<'a>(events: &Vec<Vec<StampedEvent<'a>>>) -> Vec<StampedEvent<'a>> {
        let mut ret = Vec::<StampedEvent<'a>>::new();
        for vec in events {
            let mut valid_events: Vec<StampedEvent> = vec
                .iter()
                .map(|e| *e)
                .filter(|e| match e.event.kind {
                    TrackEventKind::Meta(m) => match m {
                        MetaMessage::Tempo(_) => true,
                        MetaMessage::TimeSignature(_, _, _, _) => true,
                        _ => false,
                    },
                    _ => false,
                })
                .collect::<Vec<StampedEvent>>();

            ret.append(&mut valid_events);
        }

        ret
    }

    fn merge_events<'a>(
        mut a: Vec<StampedEvent<'a>>,
        mut b: Vec<StampedEvent<'a>>,
    ) -> Vec<StampedEvent<'a>> {
        let mut ret = Vec::<StampedEvent>::new();
        ret.append(&mut a);
        ret.append(&mut b);
        ret.sort_by(|a, b| a.ticks_elapsed.partial_cmp(&b.ticks_elapsed).unwrap());
        ret
    }

    fn is_data_track(track: &Vec<StampedEvent>) -> bool {
        for d in track {
            match d.event.kind {
                TrackEventKind::Midi { channel: _, message: _ } => {
                    return false;
                }
                _ => {}
            };
        }

        return true;
    }
}

pub struct MidiConverter<'a> {
    source: String,
    configuration: &'a Config,
}

impl<'a> MidiConverter<'a> {
    pub fn new(source: String, configuration: &'a Config) -> Self {
        MidiConverter {
            source,
            configuration,
        }
    }

    fn track_to_stamped(track: &Vec<TrackEvent<'a>>) -> Vec<StampedEvent<'a>> {
        let mut total_ticks: u64 = 0;
        let mut ret = Vec::<StampedEvent>::new();

        for event in track {
            total_ticks += event.delta.as_int() as u64;

            let stamped = StampedEvent {
                event: event.clone(),
                ticks_elapsed: total_ticks,
            };

            ret.push(stamped);
        }

        ret
    }

    pub fn to_root_merge_notes_and_meta(&self) -> Result<Root, &'static str> {
        if let Ok(vecs) = self.to_root_merge_meta() {
            let strip_name: Vec<Root> = vecs.into_iter().map(|v| v.1).collect();
            if let Some(merged) = Root::merge_note_events_vec(&strip_name) {
                return Ok(merged);
            }
        }
        Err("Failed to merge midi tracks")
    }

    pub fn to_root_merge_meta(&self) -> Result<Vec::<(String, Root)>, &'static str> {
        let buf = std::fs::read(self.source.clone())
            .expect("Failed to read source midi file. Please make sure the path exists.");
        let smf = midly::Smf::parse(&buf).unwrap();

        let mut track_as_stamped = Vec::<Vec<StampedEvent>>::new();
        for track in smf.tracks.iter() {
            track_as_stamped.push(MidiConverter::track_to_stamped(&track));
        }

        let meta_events = StampedEvent::get_meta_events(&track_as_stamped);

        // Remove data track if it exists
        track_as_stamped = track_as_stamped
            .into_iter()
            .filter(|t| !StampedEvent::is_data_track(t))
            .collect();

        for stamped_track in track_as_stamped.iter_mut() {
            *stamped_track =
                StampedEvent::merge_events(stamped_track.to_vec(), meta_events.clone());
        }

        let mut tracks_as_offsets = Vec::<TrackAsOffsets>::new();
        for track in track_as_stamped {
            let mut offsets = Vec::<OffsetEvent>::new();

            if let Some(f) = track.first() {
                // This assumes event 0 is at tick 0. Which should be true.
                offsets.push(OffsetEvent {
                    event: f.event,
                    delta: 0,
                });
            }

            for x in 1..track.len() {
                let current = track[x];
                let prev = track[x - 1];

                offsets.push(OffsetEvent {
                    event: current.event,
                    delta: current.ticks_elapsed - prev.ticks_elapsed,
                });
            }
            
            tracks_as_offsets.push(TrackAsOffsets { offsets: offsets });
        }

        let mut roots = Vec::<(String, Root)>::new();        
        for (i, track) in tracks_as_offsets.iter().enumerate() {
            match self.track_to_root_from_offsets(track, &smf) {
                Ok(root) => {
                    let name = match track.track_name() {
                        Some(name) => name,
                        None => match i {
                            0 => "Easy",
                            1 => "Normal",
                            2 => "Hard",
                            _ => "OutOfBounds",
                        }.to_string()
                    };
                    roots.push((name, root));
                },
                Err(e) => {eprintln!("{}", e);}
            }
        }

        return Ok(roots);
    }

    fn track_to_root_from_offsets (&self, track: &TrackAsOffsets, smf: &Smf) -> Result<Root, &'static str> {
        let mut stamped_hits: Vec<Note> = vec![];
        let mut bpm_changes: Vec<Bpmchange> = vec![];
        let mut ticks_elapsed: u64 = 0;

        let mut tick_len: f64 = 0_f64;
        let mut current_beat_len: f64 = 0.0;
        let mut global_beat_accumulator: f64 = 0.0;

        for offset in track.offsets.iter() {
            ticks_elapsed = ticks_elapsed + offset.delta;
            let _time_accumulator = (ticks_elapsed as f64 * tick_len) / 1_000_000_f64;

            global_beat_accumulator += if current_beat_len != 0.0 {
                offset.delta as f64 / current_beat_len
            } else {
                0.0
            };

            match offset.event.kind {
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

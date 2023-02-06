pub mod edda_objects {
    use serde_derive::Deserialize;
    use serde_derive::Serialize;
    use serde_json::Value;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Root {
        #[serde(rename = "_version")]
        pub version: String,
        #[serde(rename = "_customData")]
        pub custom_data: CustomData,
        #[serde(rename = "_events")]
        pub events: Vec<Value>,
        #[serde(rename = "_notes")]
        pub notes: Vec<Note>,
        #[serde(rename = "_obstacles")]
        pub obstacles: Vec<Value>,
    }

    impl Root {
        pub fn merge_note_events_vec(data: &Vec<Root>) -> Option<Root> {
            if let Some(first) = data.first() {
                let mut merged = first.clone();
                let mut notes = Vec::<Note>::new();
                for d in data {
                    notes.append(&mut d.notes.clone());
                }

                notes.sort_by(|i, j| i.time.partial_cmp(&j.time).unwrap());
                merged.notes = notes;

                return Some(merged);
            } else {
                return None;
            }
        }
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CustomData {
        #[serde(rename = "_time")]
        pub time: i64,
        #[serde(rename = "_BPMChanges")]
        pub bpmchanges: Vec<Bpmchange>,
        #[serde(rename = "_bookmarks")]
        pub bookmarks: Vec<Value>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Bpmchange {
        #[serde(rename = "_BPM")]
        pub bpm: f64,
        #[serde(rename = "_time")]
        pub time: f64,
        #[serde(rename = "_beatsPerBar")]
        pub beats_per_bar: i64,
        #[serde(rename = "_metronomeOffset")]
        pub metronome_offset: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Note {
        #[serde(rename = "_time")]
        pub time: f64,
        #[serde(rename = "_lineIndex")]
        pub line_index: i64,
        #[serde(rename = "_lineLayer")]
        pub line_layer: i64,
        #[serde(rename = "_type")]
        pub type_field: i64,
        #[serde(rename = "_cutDirection")]
        pub cut_direction: i64,
    }
}

pub mod custom {
    use serde_derive::{Deserialize, Serialize};

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Config {
        pub drum_map: Vec<u8>,
    }
}

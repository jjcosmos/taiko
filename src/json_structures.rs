pub mod edda_info {
    use serde_derive::Deserialize;
    use serde_derive::Serialize;
    use serde_json::Value;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Root {
        #[serde(rename = "_version")]
        pub version: String,
        #[serde(rename = "_songName")]
        pub song_name: String,
        #[serde(rename = "_songSubName")]
        pub song_sub_name: String,
        #[serde(rename = "_songAuthorName")]
        pub song_author_name: String,
        #[serde(rename = "_levelAuthorName")]
        pub level_author_name: String,
        #[serde(rename = "_explicit")]
        pub explicit: String,
        #[serde(rename = "_beatsPerMinute")]
        pub beats_per_minute: f64,
        #[serde(rename = "_shuffle")]
        pub shuffle: i64,
        #[serde(rename = "_shufflePeriod")]
        pub shuffle_period: f64,
        #[serde(rename = "_previewStartTime")]
        pub preview_start_time: i64,
        #[serde(rename = "_previewDuration")]
        pub preview_duration: i64,
        #[serde(rename = "_songApproximativeDuration")]
        pub song_approximative_duration: i64,
        #[serde(rename = "_songFilename")]
        pub song_filename: String,
        #[serde(rename = "_coverImageFilename")]
        pub cover_image_filename: String,
        #[serde(rename = "_environmentName")]
        pub environment_name: String,
        #[serde(rename = "_songTimeOffset")]
        pub song_time_offset: i64,
        #[serde(rename = "_customData")]
        pub custom_data: CustomData,
        #[serde(rename = "_difficultyBeatmapSets")]
        pub difficulty_beatmap_sets: Vec<DifficultyBeatmapSet>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CustomData {
        #[serde(rename = "_contributors")]
        pub contributors: Vec<Value>,
        #[serde(rename = "_editors")]
        pub editors: Editors,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Editors {
        #[serde(rename = "Edda")]
        pub edda: Edda,
        #[serde(rename = "_lastEditedBy")]
        pub last_edited_by: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Edda {
        pub version: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DifficultyBeatmapSet {
        #[serde(rename = "_beatmapCharacteristicName")]
        pub beatmap_characteristic_name: String,
        #[serde(rename = "_difficultyBeatmaps")]
        pub difficulty_beatmaps: Vec<DifficultyBeatmap>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DifficultyBeatmap {
        #[serde(rename = "_difficulty")]
        pub difficulty: String,
        #[serde(rename = "_difficultyRank")]
        pub difficulty_rank: i64,
        #[serde(rename = "_beatmapFilename")]
        pub beatmap_filename: String,
        #[serde(rename = "_noteJumpMovementSpeed")]
        pub note_jump_movement_speed: f64,
        #[serde(rename = "_noteJumpStartBeatOffset")]
        pub note_jump_start_beat_offset: i64,
        #[serde(rename = "_customData")]
        pub custom_data: CustomData2,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CustomData2 {
        #[serde(rename = "_editorOffset")]
        pub editor_offset: i64,
        #[serde(rename = "_editorOldOffset")]
        pub editor_old_offset: i64,
        #[serde(rename = "_editorGridSpacing")]
        pub editor_grid_spacing: f64,
        #[serde(rename = "_editorGridDivision")]
        pub editor_grid_division: i64,
        #[serde(rename = "_warnings")]
        pub warnings: Vec<Value>,
        #[serde(rename = "_information")]
        pub information: Vec<Value>,
        #[serde(rename = "_suggestions")]
        pub suggestions: Vec<Value>,
        #[serde(rename = "_requirements")]
        pub requirements: Vec<Value>,
    }
}

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

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Config {
        pub drum_map: Vec<u8>,
        pub batch_output_extension: String,
    }

    impl Default for Config {
        fn default() -> Self {
            Config {
                drum_map: (60..64).collect(),
                batch_output_extension: ".dat".to_owned(),
            }
        }
    }
}

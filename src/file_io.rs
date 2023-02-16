use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::json_structures::custom::Config;

pub fn get_or_create_file_rw(path: &Path) -> Result<File, std::io::Error> {
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

pub fn write_output_json(path: &String, data: &crate::json_structures::edda_objects::Root) {
    let file_name = path;
    match File::create(file_name) {
        Ok(mut file) => {
            println!("Writing to {} ...", file_name);
            std::io::Write::write_all(
                &mut file,
                serde_json::to_string_pretty(data)
                    .unwrap_or_default()
                    .as_bytes(),
            )
            .expect("Failed to serialize data");
        }
        Err(e) => eprintln!("Failed to create file {}: {}", path, e),
    }
}

pub fn save_config(config: &Config, file: &mut File) {
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

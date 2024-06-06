use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::io;

pub const CONFIG_FILE: &str = "config.json";

pub fn save_configuration(
    minecraft_dir: &Option<String>,
    backup_dir: &Option<String>,
    backup_frequency: i32,
) -> io::Result<()> {
    let data = json!({
        "minecraft_directory": minecraft_dir,
        "backup_directory": backup_dir,
        "backup_frequency": backup_frequency
    });
    fs::write(CONFIG_FILE, serde_json::to_string_pretty(&data)?)
}

pub fn load_configuration() -> (Option<String>, Option<String>, i32) {
    let path = Path::new(CONFIG_FILE);
    let mut backup_frequency = 24; // Default backup frequency in hours
    let (minecraft_dir, backup_dir) = if path.exists() {
        let data = fs::read_to_string(path).unwrap();
        let json: Value = serde_json::from_str(&data).unwrap();
        let minecraft_dir = json["minecraft_directory"].as_str().map(String::from);
        let backup_dir = json["backup_directory"].as_str().map(String::from);
        if let Some(freq) = json["backup_frequency"].as_i64() {
            backup_frequency = freq as i32;
        }
        (minecraft_dir, backup_dir)
    } else {
        (None, None)
    };
    (minecraft_dir, backup_dir, backup_frequency)
}
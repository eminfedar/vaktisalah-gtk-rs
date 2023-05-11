use std::{io, path::{PathBuf}};
use once_cell::sync::Lazy;

use serde::{Serialize, Deserialize};

// === PREFERENCE LOADING & SAVING ===
static PREFERENCES_TEMPLATE:&'static str = include_str!("../data/preferences.json");

static PREFERENCES_JSON_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let pathbuf = PathBuf::from(format!("{}/io.github.eminfedar.vaktisalah-gtk-rs/preferences.json", dirs::config_dir().unwrap().to_str().unwrap()));

    if !pathbuf.exists() {
        std::fs::create_dir_all(pathbuf.parent().unwrap()).unwrap();
        std::fs::write(pathbuf.as_path(), PREFERENCES_TEMPLATE).unwrap();
    };

    pathbuf
});


#[derive(Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub country: String,
    pub city: String,
    pub district: String,
    pub district_id: String,
    pub warning_minutes: u8,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PreferencesJson {
    pub preferences: Preferences,
    pub countries: serde_json::Value,
    pub cities: serde_json::Value,
    pub districts: serde_json::Value,
    pub prayer_times: serde_json::Map<String, serde_json::Value>,
}

pub fn read_preferences_json_file() -> io::Result<PreferencesJson> {
    let preferences_str = std::fs::read_to_string(PREFERENCES_JSON_PATH.as_path())?;
    let preferences: PreferencesJson = serde_json::from_str(&preferences_str)?;

    Ok(preferences)
}

pub fn save_preferences_json(preferences: &PreferencesJson) -> Result<(), Box<dyn std::error::Error>> {
    let preferences_str = serde_json::to_string(preferences)?;

    std::fs::write(PREFERENCES_JSON_PATH.as_path(), &preferences_str)?;

    Ok(())
}

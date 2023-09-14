use once_cell::sync::Lazy;
use std::{io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::ListItemIDNameGtk;
use relm4::tokio;

// === PREFERENCE LOADING & SAVING ===
static PREFERENCES_TEMPLATE: &str = include_str!("../data/preferences.json");

static PREFERENCES_JSON_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let pathbuf = PathBuf::from(format!(
        "{}/io.github.eminfedar.vaktisalah-gtk-rs/preferences.json",
        dirs::config_dir().unwrap().to_str().unwrap()
    ));

    if !pathbuf.exists() {
        std::fs::create_dir_all(pathbuf.parent().unwrap()).unwrap();
        std::fs::write(pathbuf.as_path(), PREFERENCES_TEMPLATE).unwrap();
    };

    pathbuf
});

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    pub country: String,
    pub city: String,
    pub district: String,
    pub district_id: String,
    pub warning_minutes: u8,
    pub dark_mode: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PreferencesJson {
    pub preferences: Preferences,
    pub countries: serde_json::Value,
    pub cities: serde_json::Value,
    pub districts: serde_json::Value,
    pub prayer_times: serde_json::Map<String, serde_json::Value>,
}

impl PreferencesJson {
    pub fn value_to_listitem(key_value_map: &serde_json::Value) -> Vec<ListItemIDNameGtk> {
        key_value_map
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, id)| ListItemIDNameGtk::new(id.as_str().unwrap(), name.as_str()))
            .collect()
    }
}

pub fn read_preferences_json_file() -> io::Result<PreferencesJson> {
    let preferences_str = std::fs::read_to_string(PREFERENCES_JSON_PATH.as_path())?;
    let preferences: PreferencesJson = serde_json::from_str(preferences_str.as_str())?;

    Ok(preferences)
}

pub async fn save_preferences_json(
    preferences: &PreferencesJson,
) -> Result<(), Box<dyn std::error::Error>> {
    let preferences_str = serde_json::to_string(preferences)?;

    tokio::fs::write(PREFERENCES_JSON_PATH.as_path(), preferences_str.as_str()).await?;

    Ok(())
}

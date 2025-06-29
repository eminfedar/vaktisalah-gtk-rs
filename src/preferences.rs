use std::{cell::RefCell, collections::HashMap, fs, io};

use serde::{Deserialize, Serialize};

use crate::prayer::PrayerTimesWithDate;

// === PREFERENCE LOADING & SAVING ===
static PREFERENCES_TEMPLATE: &str = include_str!("../data/preferences.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    pub country: RefCell<String>,
    pub city: RefCell<String>,
    pub district: RefCell<String>,
    pub district_id: RefCell<String>,
    pub warning_minutes: RefCell<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreferencesJson {
    pub preferences: Preferences,
    pub countries: RefCell<HashMap<String, String>>,
    pub countries_en: RefCell<HashMap<String, String>>,
    pub cities: RefCell<HashMap<String, String>>,
    pub districts: RefCell<HashMap<String, String>>,
    pub prayer_times: RefCell<HashMap<String, PrayerTimesWithDate>>,
}

impl Default for PreferencesJson {
    fn default() -> Self {
        let mut preferences_pathbuf = gtk::glib::user_config_dir();
        preferences_pathbuf.push("io.github.eminfedar.vaktisalah-gtk-rs/preferences.json");

        if preferences_pathbuf.exists() {
            let preferences_str = std::fs::read_to_string(preferences_pathbuf.as_path()).unwrap();

            serde_json::from_str(preferences_str.as_str()).unwrap()
        } else {
            std::fs::create_dir_all(preferences_pathbuf.parent().unwrap()).unwrap();
            std::fs::write(preferences_pathbuf.as_path(), PREFERENCES_TEMPLATE).unwrap();

            serde_json::from_str(PREFERENCES_TEMPLATE).unwrap()
        }
    }
}

impl PreferencesJson {
    pub fn save(&self) -> io::Result<()> {
        let mut preferences_pathbuf = gtk::glib::user_config_dir();
        preferences_pathbuf.push("io.github.eminfedar.vaktisalah-gtk-rs/preferences.json");

        fs::write(preferences_pathbuf, serde_json::to_string(self)?)?;

        Ok(())
    }
}

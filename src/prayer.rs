use std::fmt::Display;

use crate::preferences::PreferencesJson;
use chrono::{Days, Local, Utc};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Prayer {
    Fajr = 0,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
    FajrNextDay,
}

impl Default for Prayer {
    fn default() -> Self {
        Prayer::Fajr
    }
}

impl From<u8> for Prayer {
    fn from(value: u8) -> Self {
        match value {
            0 => Prayer::Fajr,
            1 => Prayer::Sunrise,
            2 => Prayer::Dhuhr,
            3 => Prayer::Asr,
            4 => Prayer::Maghrib,
            5 => Prayer::Isha,
            6 => Prayer::FajrNextDay,
            _ => Prayer::Fajr,
        }
    }
}

impl Display for Prayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = match self {
            Prayer::Fajr => "to Fajr",
            Prayer::Sunrise => "to Sunrise",
            Prayer::Dhuhr => "to Dhuhr",
            Prayer::Asr => "to Asr",
            Prayer::Maghrib => "to Maghrib",
            Prayer::Isha => "to Isha",
            Prayer::FajrNextDay => "to FajrNextDay",
        };

        write!(f, "{}", p)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RemainingTime {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub next_prayer: Prayer,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrayerTimesWithDate {
    pub Imsak: String,
    pub Gunes: String,
    pub Ogle: String,
    pub Ikindi: String,
    pub Aksam: String,
    pub Yatsi: String,

    pub MiladiTarihKisa: String,
    pub HicriTarihKisa: String,
    pub HicriTarihUzun: String,
}

pub fn get_prayer_times_with_date(
    preferences_json: &PreferencesJson,
    additional_day: u64,
) -> Option<PrayerTimesWithDate> {
    let date_formatted = Utc::now()
        .checked_add_days(Days::new(additional_day))?
        .format("%d.%m.%Y")
        .to_string();

    serde_json::from_value(
        preferences_json
            .prayer_times
            .get(&date_formatted)?
            .to_owned(),
    )
    .ok()
}

pub fn is_prayer_times_valid(preferences: &PreferencesJson) -> bool {
    let today = Utc::now();
    let tomorrow = Utc::now().checked_add_days(Days::new(1)).unwrap();

    let today_formatted = today.format("%d.%m.%Y").to_string();
    let tomorrow_formatted = tomorrow.format("%d.%m.%Y").to_string();

    if preferences.prayer_times.get(&today_formatted).is_none()
        || preferences.prayer_times.get(&tomorrow_formatted).is_none()
    {
        return false;
    }

    true
}

pub fn calculate_remaining_time(
    todays_prayers: &Option<PrayerTimesWithDate>,
    tomorrows_prayers: &Option<PrayerTimesWithDate>,
) -> Option<RemainingTime> {
    let (todays_prayers, tomorrows_prayers) = match (todays_prayers, tomorrows_prayers) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            return None;
        }
    };

    // Calculate Remaning Time
    let today_prayer_times_array = [
        &todays_prayers.Imsak,
        &todays_prayers.Gunes,
        &todays_prayers.Ogle,
        &todays_prayers.Ikindi,
        &todays_prayers.Aksam,
        &todays_prayers.Yatsi,
        &tomorrows_prayers.Imsak,
    ];
    let now = Local::now();

    for (i, prayer_time) in today_prayer_times_array.iter().enumerate() {
        let mut hours = (prayer_time[0..2]).parse::<i32>().unwrap();
        let minutes = (prayer_time[3..5]).parse::<i32>().unwrap();

        if i == 6 {
            hours += 24;
        }

        let now_formatted = now.time().format("%H:%M:%S").to_string();
        let now_hours = (now_formatted[0..2]).parse::<i32>().unwrap();
        let now_minutes = (now_formatted[3..5]).parse::<i32>().unwrap();
        let now_seconds = (now_formatted[6..8]).parse::<i32>().unwrap();

        if now_hours > hours {
            continue;
        }

        if now_hours == hours && now_minutes >= minutes {
            continue;
        }

        // Found the next prayer. Calculate remaining time:
        let total_now_seconds = now_hours * 3600 + now_minutes * 60 + now_seconds;
        let total_prayer_seconds = hours * 3600 + minutes * 60;
        let total_remaining_seconds = total_prayer_seconds - total_now_seconds;

        let remaining_seconds = total_remaining_seconds % 60;
        let remaining_minutes = (total_remaining_seconds / 60) % 60;
        let remaining_hours = total_remaining_seconds / 3600;

        return Some(RemainingTime {
            hours: remaining_hours as u8,
            minutes: remaining_minutes as u8,
            seconds: remaining_seconds as u8,
            next_prayer: Prayer::from(i as u8),
        });
    }

    Some(RemainingTime {
        hours: 0,
        minutes: 0,
        seconds: 0,
        next_prayer: Prayer::Fajr,
    })
}

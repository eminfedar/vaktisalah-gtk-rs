
use chrono::{Utc, Days};
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::preferences::PreferencesJson;

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrayerTimesWithDate {
    pub Imsak: String,
    pub Gunes: String,
    pub Ogle: String,
    pub Ikindi: String,
    pub Aksam: String,
    pub Yatsi: String,

    pub MiladiTarihKisa: String,
    pub HicriTarihKisa: String,
    pub HicriTarihUzun: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct DistrictResponse {
    pub IlceAdi: String,
    pub IlceAdiEn: String,
    pub IlceID: String
}

/// Basic HTTP Request
fn get_request(endpoint: &str, id: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let url = format!("http://ezanvakti.herokuapp.com/{}/{}", endpoint, id);
    
    reqwest::blocking::get(url)
}

/// Get Prayer Times from internet
fn get_prayer_times(district_id: &str) -> Result<Vec<PrayerTimesWithDate>, reqwest::Error> {
    get_request("vakitler", district_id)?.json::<Vec<PrayerTimesWithDate>>()
}
/// Get City List from internet
pub fn get_city_list(country_id: &str) -> Result<serde_json::Value, reqwest::Error> {
    get_request("sehirler", country_id)?.json::<serde_json::Value>()
}
/// Get District List from internet
pub fn get_district_list(city_id: &str) -> Result<serde_json::Value, reqwest::Error> {
    let response: Vec<DistrictResponse> = get_request("ilceler", city_id)?.json()?;

    let mut json_obj = json!({});
    let json_obj_mut = json_obj.as_object_mut().unwrap();
    for d in response {
        json_obj_mut.insert(d.IlceAdi, serde_json::to_value(&d.IlceID).unwrap());
    }

    Ok(json_obj)
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

pub fn update_prayer_times_on_network(
    preferences: &mut PreferencesJson,
) -> Result<(), Box<dyn std::error::Error>> {
    let monthly_prayer_times = get_prayer_times(&preferences.preferences.district_id)?;

    preferences.prayer_times.clear();
    for day in monthly_prayer_times {
        let key = day.MiladiTarihKisa.clone();
        let day_as_value = serde_json::to_value(day)?;

        preferences.prayer_times.insert(key, day_as_value);
    }

    Ok(())
}
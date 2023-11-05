use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{prayer::PrayerTimesWithDate, preferences::PreferencesJson, USER_LOCALE};

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct CityResponse {
    pub SehirAdi: String,
    pub SehirAdiEn: String,
    pub SehirID: String,
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct DistrictResponse {
    pub IlceAdi: String,
    pub IlceAdiEn: String,
    pub IlceID: String,
}

/// Basic HTTP Request
async fn get_request(endpoint: &str, id: &str) -> Result<reqwest::Response, reqwest::Error> {
    let url = format!("http://ezanvakti.herokuapp.com/{}/{}", endpoint, id);

    println!("GET request to url: {url:?}");

    reqwest::get(url).await
}

/// Get Prayer Times from internet
async fn get_prayer_times(district_id: &str) -> Result<Vec<PrayerTimesWithDate>, reqwest::Error> {
    get_request("vakitler", district_id)
        .await?
        .json::<Vec<PrayerTimesWithDate>>()
        .await
}
/// Get City List from internet
pub async fn get_city_list(country_id: &str) -> Result<serde_json::Value, reqwest::Error> {
    let response: Vec<CityResponse> = get_request("sehirler", country_id).await?.json().await?;

    let mut json_obj = json!({});

    if USER_LOCALE.as_str() == "tr-TR" {
        for c in response {
            json_obj[c.SehirAdi] = serde_json::to_value(&c.SehirID).unwrap();
        }
    } else {
        for c in response {
            json_obj[c.SehirAdiEn] = serde_json::to_value(&c.SehirID).unwrap();
        }
    }

    Ok(json_obj)
}
/// Get District List from internet
pub async fn get_district_list(city_id: &str) -> Result<serde_json::Value, reqwest::Error> {
    let response: Vec<DistrictResponse> = get_request("ilceler", city_id).await?.json().await?;

    let mut json_obj = json!({});

    if USER_LOCALE.as_str() == "tr-TR" {
        for d in response {
            json_obj[d.IlceAdi] = serde_json::to_value(&d.IlceID).unwrap();
        }
    } else {
        for d in response {
            json_obj[d.IlceAdiEn] = serde_json::to_value(&d.IlceID).unwrap();
        }
    }

    Ok(json_obj)
}

pub async fn update_prayer_times_on_network(
    preferences: &mut PreferencesJson,
) -> Result<(), Box<dyn std::error::Error>> {
    let monthly_prayer_times = get_prayer_times(&preferences.preferences.district_id).await?;

    preferences.prayer_times.clear();
    for day in monthly_prayer_times {
        let key = day.MiladiTarihKisa.clone();
        let day_as_value = serde_json::to_value(day)?;

        preferences.prayer_times.insert(key, day_as_value);
    }

    Ok(())
}

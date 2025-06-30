use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{current_locale, prayer::PrayerTimesWithDate};

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
    let url = format!("http://ezanvakti.emushaf.net/{}/{}", endpoint, id);

    println!("GET: {url:?}");

    reqwest::get(url).await
}

/// Get Prayer Times from internet
pub async fn get_prayer_times(
    district_id: &str,
) -> Result<Vec<PrayerTimesWithDate>, reqwest::Error> {
    get_request("vakitler", district_id)
        .await?
        .json::<Vec<PrayerTimesWithDate>>()
        .await
}
/// Get City List from internet
pub async fn get_city_list(country_id: &str) -> Result<HashMap<String, String>, reqwest::Error> {
    let response: Vec<CityResponse> = get_request("sehirler", country_id).await?.json().await?;

    let mut hm = HashMap::new();

    let locale = current_locale::current_locale();

    if locale == "tr-TR" {
        for c in response {
            hm.insert(c.SehirAdi, c.SehirID);
        }
    } else {
        for c in response {
            hm.insert(c.SehirAdiEn, c.SehirID);
        }
    }

    Ok(hm)
}
/// Get District List from internet
pub async fn get_district_list(city_id: &str) -> Result<HashMap<String, String>, reqwest::Error> {
    let response: Vec<DistrictResponse> = get_request("ilceler", city_id).await?.json().await?;

    let mut hm = HashMap::new();

    let locale = current_locale::current_locale();

    if locale == "tr-TR" {
        for d in response {
            hm.insert(d.IlceAdi, d.IlceID);
        }
    } else {
        for d in response {
            hm.insert(d.IlceAdiEn, d.IlceID);
        }
    }

    Ok(hm)
}

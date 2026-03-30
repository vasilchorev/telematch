use crate::Lang;
use serde::Deserialize;
use std::error::Error;

type GeoResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct CityCandidate {
    pub label: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone)]
pub struct Geocoder {
    client: reqwest::Client,
    api_key: String,
}

impl Geocoder {
    pub fn new() -> GeoResult<Self> {
        let client = reqwest::Client::builder().build()?;
        let api_key = std::env::var("GOOGLE_MAPS_API_KEY")?;

        Ok(Self { client, api_key })
    }

    pub async fn search_cities(&self, query: &str, lang: Lang) -> GeoResult<Vec<CityCandidate>> {
        let response = self
            .client
            .get("https://maps.googleapis.com/maps/api/geocode/json")
            .query(&[
                ("address", query),
                ("language", lang.as_db_code()),
                ("key", self.api_key.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<GeocodeResponse>()
            .await?;

        match response.status.as_str() {
            "OK" => {}
            "ZERO_RESULTS" => return Ok(Vec::new()),
            other => {
                let message = response
                    .error_message
                    .unwrap_or_else(|| format!("google geocoding failed with status {other}"));
                return Err(message.into());
            }
        }

        let candidates = response
            .results
            .into_iter()
            .filter(|result| is_city_like(&result.types))
            .map(|result| CityCandidate {
                label: extract_city_label(&result),
                latitude: result.geometry.location.lat,
                longitude: result.geometry.location.lng,
            })
            .collect();

        Ok(candidates)
    }

    pub async fn reverse_geocode_city(
        &self,
        latitude: f64,
        longitude: f64,
        lang: Lang,
    ) -> GeoResult<Option<CityCandidate>> {
        let latlng = format!("{latitude},{longitude}");

        let response = self
            .client
            .get("https://maps.googleapis.com/maps/api/geocode/json")
            .query(&[
                ("latlng", latlng.as_str()),
                ("language", lang.as_db_code()),
                ("key", self.api_key.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<GeocodeResponse>()
            .await?;

        match response.status.as_str() {
            "OK" => {}
            "ZERO_RESULTS" => return Ok(None),
            other => {
                let message = response
                    .error_message
                    .unwrap_or_else(|| format!("google reverse geocoding failed with status {other}"));
                return Err(message.into());
            }
        }

        let candidate = response
            .results
            .into_iter()
            .find(|result| is_city_like(&result.types))
            .map(|result| CityCandidate {
                label: extract_city_label(&result),
                latitude,
                longitude,
            });

        Ok(candidate)
    }
}

fn is_city_like(types: &[String]) -> bool {
    types.iter().any(|t| t == "locality")
        || types.iter().any(|t| t == "postal_town")
        || types.iter().any(|t| t == "administrative_area_level_3")
}

fn extract_city_label(result: &GeocodeResult) -> String {
    result
        .address_components
        .iter()
        .find(|component| component.types.iter().any(|t| t == "locality"))
        .map(|component| component.long_name.clone())
        .or_else(|| {
            result
                .address_components
                .iter()
                .find(|component| component.types.iter().any(|t| t == "postal_town"))
                .map(|component| component.long_name.clone())
        })
        .or_else(|| {
            result
                .address_components
                .iter()
                .find(|component| {
                    component
                        .types
                        .iter()
                        .any(|t| t == "administrative_area_level_3")
                })
                .map(|component| component.long_name.clone())
        })
        .unwrap_or_else(|| result.formatted_address.clone())
}

#[derive(Debug, Deserialize)]
struct GeocodeResponse {
    status: String,
    results: Vec<GeocodeResult>,
    error_message: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct GeocodeResult {
    formatted_address: String,
    address_components: Vec<AddressComponent>,
    types: Vec<String>,
    geometry: Geometry,
}

#[derive(Debug, Deserialize, Clone)]
struct AddressComponent {
    long_name: String,
    types: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Geometry {
    location: LatLng,
}

#[derive(Debug, Deserialize, Clone)]
struct LatLng {
    lat: f64,
    lng: f64,
}

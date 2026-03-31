use crate::Language;
use serde::Deserialize;
use std::env::VarError;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

type GeoResult<T> = Result<T, GeocodingError>;

const GEOCODE_ENDPOINT: &str = "https://maps.googleapis.com/maps/api/geocode/json";
const CITY_LIKE_TYPES: [&str; 3] = ["locality", "postal_town", "administrative_area_level_3"];

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

#[derive(Debug)]
pub enum GeocodingError {
    MissingApiKey(VarError),
    Http(reqwest::Error),
    Api {
        operation: &'static str,
        status: String,
        message: Option<String>,
    },
}

impl Display for GeocodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey(_) => write!(f, "GOOGLE_MAPS_API_KEY is not set"),
            Self::Http(error) => write!(f, "geocoding request failed: {error}"),
            Self::Api {
                operation,
                status,
                message,
            } => {
                if let Some(message) = message {
                    write!(f, "{operation} failed with status {status}: {message}")
                } else {
                    write!(f, "{operation} failed with status {status}")
                }
            }
        }
    }
}

impl Error for GeocodingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingApiKey(error) => Some(error),
            Self::Http(error) => Some(error),
            Self::Api { .. } => None,
        }
    }
}

impl Geocoder {
    pub fn new() -> GeoResult<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(GeocodingError::Http)?;
        let api_key =
            std::env::var("GOOGLE_MAPS_API_KEY").map_err(GeocodingError::MissingApiKey)?;

        Ok(Self { client, api_key })
    }

    pub async fn search_cities(
        &self,
        query: &str,
        language: Language,
    ) -> GeoResult<Vec<CityCandidate>> {
        self.fetch_results(
            &[
                ("address", query),
                ("language", language.as_db_code()),
                ("key", self.api_key.as_str()),
            ],
            "google geocoding",
        )
        .await
        .map(|results| {
            results
                .into_iter()
                .filter(GeocodeResult::is_city_like)
                .map(GeocodeResult::into_city_candidate)
                .collect()
        })
    }

    pub async fn reverse_geocode_city(
        &self,
        latitude: f64,
        longitude: f64,
        language: Language,
    ) -> GeoResult<Option<CityCandidate>> {
        let latlng = format!("{latitude},{longitude}");

        let candidate = self
            .fetch_results(
                &[
                    ("latlng", latlng.as_str()),
                    ("language", language.as_db_code()),
                    ("key", self.api_key.as_str()),
                ],
                "google reverse geocoding",
            )
            .await?
            .into_iter()
            .find(GeocodeResult::is_city_like)
            .map(|result| CityCandidate {
                label: result.city_label(),
                latitude,
                longitude,
            });

        Ok(candidate)
    }

    async fn fetch_results(
        &self,
        query: &[(&str, &str)],
        operation: &'static str,
    ) -> GeoResult<Vec<GeocodeResult>> {
        let response = self
            .client
            .get(GEOCODE_ENDPOINT)
            .query(query)
            .send()
            .await
            .map_err(GeocodingError::Http)?
            .error_for_status()
            .map_err(GeocodingError::Http)?
            .json::<GeocodeResponse>()
            .await
            .map_err(GeocodingError::Http)?;

        match response.status.as_str() {
            "OK" => Ok(response.results),
            "ZERO_RESULTS" => Ok(Vec::new()),
            _ => Err(GeocodingError::Api {
                operation,
                status: response.status,
                message: response.error_message,
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GeocodeResponse {
    status: String,
    results: Vec<GeocodeResult>,
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeocodeResult {
    formatted_address: String,
    address_components: Vec<AddressComponent>,
    types: Vec<String>,
    geometry: Geometry,
}

impl GeocodeResult {
    fn is_city_like(&self) -> bool {
        CITY_LIKE_TYPES
            .into_iter()
            .any(|city_type| self.has_type(city_type))
    }

    fn city_label(&self) -> String {
        CITY_LIKE_TYPES
            .into_iter()
            .find_map(|city_type| {
                self.address_components
                    .iter()
                    .find(|component| component.has_type(city_type))
                    .map(|component| component.long_name.clone())
            })
            .unwrap_or_else(|| self.formatted_address.clone())
    }

    fn into_city_candidate(self) -> CityCandidate {
        let label = self.city_label();

        CityCandidate {
            label,
            latitude: self.geometry.location.lat,
            longitude: self.geometry.location.lng,
        }
    }

    fn has_type(&self, expected_type: &str) -> bool {
        self.types
            .iter()
            .any(|actual_type| actual_type == expected_type)
    }
}

#[derive(Debug, Deserialize)]
struct AddressComponent {
    long_name: String,
    types: Vec<String>,
}

impl AddressComponent {
    fn has_type(&self, expected_type: &str) -> bool {
        self.types
            .iter()
            .any(|actual_type| actual_type == expected_type)
    }
}

#[derive(Debug, Deserialize)]
struct Geometry {
    location: LatLng,
}

#[derive(Debug, Deserialize)]
struct LatLng {
    lat: f64,
    lng: f64,
}

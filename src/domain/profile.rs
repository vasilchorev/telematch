use crate::Language;
use crate::telegram::i18n::TextKey;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::ops::RangeInclusive;

const VALID_AGE_RANGE: RangeInclusive<u8> = 1..=100;

#[derive(Debug, Clone, Default)]
pub struct Profile {
    pub telegram_user_id: Option<i64>,
    pub chat_id: Option<i64>,
    pub username: Option<String>,
    pub language_code: Option<String>,
    pub name: Option<String>,
    pub gender: Option<Gender>,
    pub looking_for: Option<Gender>,
    pub age: Option<u8>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub photo: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Profile {
    pub fn display_text(&self) -> String {
        let name = self.name.as_deref().unwrap_or("Unknown");
        let age = self
            .age
            .map_or_else(|| "?".to_string(), |age| age.to_string());
        let location = self.location.as_deref().unwrap_or("Unknown location");
        let description = self
            .description
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("No bio provided.");

        format!("{name}, {age}, {location} - {description}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompleteProfile {
    pub telegram_user_id: i64,
    pub chat_id: i64,
    pub username: Option<String>,
    pub language_code: String,
    pub name: String,
    pub gender: Gender,
    pub looking_for: Gender,
    pub age: u8,
    pub location: String,
    pub description: String,
    pub photo: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl TryFrom<&Profile> for CompleteProfile {
    type Error = ProfileValidationError;

    fn try_from(profile: &Profile) -> Result<Self, Self::Error> {
        let age = required_value(profile.age, ProfileValidationError::MissingAge)?;
        if !VALID_AGE_RANGE.contains(&age) {
            return Err(ProfileValidationError::InvalidAge(age));
        }

        Ok(Self {
            telegram_user_id: required_value(
                profile.telegram_user_id,
                ProfileValidationError::MissingTelegramUserId,
            )?,
            chat_id: required_value(profile.chat_id, ProfileValidationError::MissingChatId)?,
            username: profile.username.clone(),
            language_code: required_non_empty_text(
                profile.language_code.as_deref(),
                ProfileValidationError::MissingLanguageCode,
            )?,
            name: required_non_empty_text(
                profile.name.as_deref(),
                ProfileValidationError::MissingName,
            )?,
            gender: required_value(profile.gender, ProfileValidationError::MissingGender)?,
            looking_for: required_value(
                profile.looking_for,
                ProfileValidationError::MissingLookingFor,
            )?,
            age,
            location: required_non_empty_text(
                profile.location.as_deref(),
                ProfileValidationError::MissingLocation,
            )?,
            description: profile
                .description
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .to_owned(),
            photo: required_non_empty_text(
                profile.photo.as_deref(),
                ProfileValidationError::MissingPhoto,
            )?,
            latitude: required_value(profile.latitude, ProfileValidationError::MissingLocation)?,
            longitude: required_value(profile.longitude, ProfileValidationError::MissingLocation)?,
        })
    }
}

fn required_value<T>(
    value: Option<T>,
    error: ProfileValidationError,
) -> Result<T, ProfileValidationError> {
    value.ok_or(error)
}

fn required_non_empty_text(
    value: Option<&str>,
    error: ProfileValidationError,
) -> Result<String, ProfileValidationError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or(error)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileValidationError {
    MissingName,
    MissingGender,
    MissingLookingFor,
    MissingAge,
    InvalidAge(u8),
    MissingLocation,
    MissingPhoto,
    MissingTelegramUserId,
    MissingChatId,
    MissingLanguageCode,
}

impl Display for ProfileValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTelegramUserId => write!(f, "telegram user id is missing"),
            Self::MissingChatId => write!(f, "chat id is missing"),
            Self::MissingLanguageCode => write!(f, "language code is missing"),
            Self::MissingName => write!(f, "name is missing"),
            Self::MissingGender => write!(f, "gender is missing"),
            Self::MissingLookingFor => write!(f, "search preference is missing"),
            Self::MissingAge => write!(f, "age is missing"),
            Self::InvalidAge(age) => write!(f, "age {age} is outside the allowed range"),
            Self::MissingLocation => write!(f, "location is missing"),
            Self::MissingPhoto => write!(f, "photo is missing"),
        }
    }
}

impl Error for ProfileValidationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    pub fn from_text(text: &str, language: Language) -> Option<Self> {
        match text.trim() {
            value if value == language.text(TextKey::Male) => Some(Self::Male),
            value if value == language.text(TextKey::Female) => Some(Self::Female),
            _ => None,
        }
    }

    pub fn from_db_code(code: &str) -> Option<Self> {
        match code {
            "M" => Some(Self::Male),
            "F" => Some(Self::Female),
            _ => None,
        }
    }

    pub const fn as_db_code(self) -> &'static str {
        match self {
            Self::Male => "M",
            Self::Female => "F",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_profile() -> Profile {
        Profile {
            telegram_user_id: Some(1),
            chat_id: Some(2),
            username: Some("user".to_owned()),
            language_code: Some("en".to_owned()),
            name: Some("Alice".to_owned()),
            gender: Some(Gender::Female),
            looking_for: Some(Gender::Male),
            age: Some(25),
            location: Some("Bratislava".to_owned()),
            description: Some("  Bio  ".to_owned()),
            photo: Some("photo-id".to_owned()),
            latitude: Some(48.1486),
            longitude: Some(17.1077),
        }
    }

    #[test]
    fn complete_profile_trims_required_text_fields() {
        let mut profile = valid_profile();
        profile.name = Some("  Alice  ".to_owned());
        profile.location = Some("  Bratislava  ".to_owned());

        let complete = CompleteProfile::try_from(&profile).expect("profile should be valid");

        assert_eq!(complete.name, "Alice");
        assert_eq!(complete.location, "Bratislava");
        assert_eq!(complete.description, "Bio");
    }

    #[test]
    fn complete_profile_rejects_invalid_age() {
        let mut profile = valid_profile();
        profile.age = Some(0);

        assert_eq!(
            CompleteProfile::try_from(&profile),
            Err(ProfileValidationError::InvalidAge(0))
        );
    }
}

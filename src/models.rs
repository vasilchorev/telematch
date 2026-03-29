use crate::Lang;
use crate::bot::i18n::TextKey;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

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
}

impl Profile {
    pub fn display_text(&self) -> String {
        let name = self.name.as_deref().unwrap_or("Unknown");
        let age = self
            .age
            .map(|age| age.to_string())
            .unwrap_or_else(|| "?".to_string());
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl TryFrom<&Profile> for CompleteProfile {
    type Error = ProfileValidationError;

    fn try_from(profile: &Profile) -> Result<Self, Self::Error> {
        let age = profile.age.ok_or(ProfileValidationError::MissingAge)?;
        if !(1..=100).contains(&age) {
            return Err(ProfileValidationError::InvalidAge(age));
        }

        Ok(Self {
            telegram_user_id: profile
                .telegram_user_id
                .ok_or(ProfileValidationError::MissingTelegramUserId)?,
            chat_id: profile
                .chat_id
                .ok_or(ProfileValidationError::MissingChatId)?,
            username: profile.username.clone(),
            language_code: required_text(
                profile.language_code.as_deref(),
                ProfileValidationError::MissingLanguageCode,
            )?,
            name: required_text(profile.name.as_deref(), ProfileValidationError::MissingName)?,
            gender: profile
                .gender
                .ok_or(ProfileValidationError::MissingGender)?,
            looking_for: profile
                .looking_for
                .ok_or(ProfileValidationError::MissingLookingFor)?,
            age,
            location: required_text(
                profile.location.as_deref(),
                ProfileValidationError::MissingLocation,
            )?,
            description: required_text(
                profile.description.as_deref(),
                ProfileValidationError::MissingDescription,
            )?,
            photo: required_text(
                profile.photo.as_deref(),
                ProfileValidationError::MissingPhoto,
            )?,
        })
    }
}

fn required_text(
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
    MissingDescription,
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
            Self::MissingDescription => write!(f, "bio is missing"),
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
    pub fn from_text(text: &str, lang: Lang) -> Option<Self> {
        let text = text.trim();

        if text == lang.text(TextKey::Male) {
            Some(Self::Male)
        } else if text == lang.text(TextKey::Female) {
            Some(Self::Female)
        } else {
            None
        }
    }

    pub fn from_db_code(code: &str) -> Option<Self> {
        match code {
            "M" => Some(Self::Male),
            "F" => Some(Self::Female),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> &'static str {
        match self {
            Self::Male => "M",
            Self::Female => "F",
        }
    }
}

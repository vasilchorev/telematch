use crate::domain::Profile;
use crate::telegram::i18n::TextKey;
use crate::telegram::keyboards::{
    EN_INCOMING_LIKE_SHOW_BUTTON_TEXT, EN_INCOMING_LIKE_STOP_BUTTON_TEXT,
    ENGLISH_LANGUAGE_BUTTON_TEXT, SK_INCOMING_LIKE_SHOW_BUTTON_TEXT,
    SK_INCOMING_LIKE_STOP_BUTTON_TEXT, SLOVAK_LANGUAGE_BUTTON_TEXT,
    UK_INCOMING_LIKE_SHOW_BUTTON_TEXT, UK_INCOMING_LIKE_STOP_BUTTON_TEXT,
    UKRAINIAN_LANGUAGE_BUTTON_TEXT,
};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type AppDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = AppResult<()>;

pub const LANGUAGE_PROMPT: &str = "Choose language / Vyber jazyk / Обери мову";
pub const REVEAL_SPINNER_TEXT: &str = "✨🔍";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Sk,
    Uk,
}

impl Language {
    pub fn from_text(text: &str) -> Option<Self> {
        match text.trim() {
            ENGLISH_LANGUAGE_BUTTON_TEXT => Some(Self::En),
            SLOVAK_LANGUAGE_BUTTON_TEXT => Some(Self::Sk),
            UKRAINIAN_LANGUAGE_BUTTON_TEXT => Some(Self::Uk),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Sk => "sk",
            Self::Uk => "uk",
        }
    }

    pub fn from_db_code(code: &str) -> Self {
        match code {
            "sk" => Self::Sk,
            "uk" => Self::Uk,
            _ => Self::En,
        }
    }
}

pub fn profile_language(profile: &Profile) -> Language {
    profile
        .language_code
        .as_deref()
        .map(Language::from_db_code)
        .unwrap_or(Language::En)
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    WaitingForLanguage,
    WaitingForName {
        draft: Profile,
    },
    WaitingForGender {
        draft: Profile,
    },
    WaitingForLookingFor {
        draft: Profile,
    },
    WaitingForAge {
        draft: Profile,
    },
    WaitingForLocation {
        draft: Profile,
    },
    WaitingForLocationChoice {
        draft: Profile,
        candidates: Vec<crate::services::CityCandidate>,
    },
    WaitingForDescription {
        draft: Profile,
    },
    WaitingForDescriptionEdit {
        draft: Profile,
    },
    WaitingForPhoto {
        draft: Profile,
    },
    WaitingForPhotoEdit {
        draft: Profile,
    },
    ConfirmProfile {
        draft: Profile,
    },
    MainMenu {
        profile: Profile,
    },
    EditMenu {
        profile: Profile,
    },
    SettingsMenu {
        profile: Profile,
    },
    WaitingForLanguagePreference {
        profile: Profile,
    },
    AwaitingIncomingLikeDecision {
        profile: Profile,
        incoming_like_user_id: i64,
        pending_like_count: i64,
        target_kind: IncomingLikeTargetKind,
    },
    AwaitingProfileAction {
        profile: Profile,
        displayed_profile_user_id: i64,
        return_to_main_menu: bool,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct SenderInfo {
    pub(crate) telegram_user_id: i64,
    pub(crate) telegram_username: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationAction {
    SaveProfile,
    EditProfile,
}

impl ConfirmationAction {
    pub fn parse(text: &str) -> Option<Self> {
        if matches_text_key_in_any_language(text, TextKey::SaveProfile) {
            Some(Self::SaveProfile)
        } else if matches_text_key_in_any_language(text, TextKey::EditProfile) {
            Some(Self::EditProfile)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingLikeDecision {
    Show,
    StopViewing,
}

impl IncomingLikeDecision {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            EN_INCOMING_LIKE_SHOW_BUTTON_TEXT
            | SK_INCOMING_LIKE_SHOW_BUTTON_TEXT
            | UK_INCOMING_LIKE_SHOW_BUTTON_TEXT => Some(Self::Show),
            EN_INCOMING_LIKE_STOP_BUTTON_TEXT
            | SK_INCOMING_LIKE_STOP_BUTTON_TEXT
            | UK_INCOMING_LIKE_STOP_BUTTON_TEXT => Some(Self::StopViewing),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingLikeTargetKind {
    IncomingLike,
    PendingMutualMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMenuAction {
    EditProfile,
    ChangePhoto,
    ChangeBio,
    BackToMainMenu,
}

impl EditMenuAction {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "1" => Some(Self::EditProfile),
            "2" => Some(Self::ChangePhoto),
            "3" => Some(Self::ChangeBio),
            "4" => Some(Self::BackToMainMenu),
            _ if matches_text_key_in_any_language(text, TextKey::EditProfileMenuAction) => {
                Some(Self::EditProfile)
            }
            _ if matches_text_key_in_any_language(text, TextKey::ChangePhoto) => {
                Some(Self::ChangePhoto)
            }
            _ if matches_text_key_in_any_language(text, TextKey::ChangeBio) => {
                Some(Self::ChangeBio)
            }
            _ if matches_text_key_in_any_language(text, TextKey::BackToMainMenu) => {
                Some(Self::BackToMainMenu)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAction {
    ChangeLanguage,
    BackToMainMenu,
}

impl SettingsAction {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            _ if matches_text_key_in_any_language(text, TextKey::ChangeLanguage) => {
                Some(Self::ChangeLanguage)
            }
            _ if matches_text_key_in_any_language(text, TextKey::BackToMainMenu) => {
                Some(Self::BackToMainMenu)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuAction {
    ViewProfiles,
    MyProfile,
    Settings,
    DeactivateProfile,
}

impl MainMenuAction {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            // fallback
            "1" => Some(Self::ViewProfiles),
            "2" => Some(Self::MyProfile),
            "3" => Some(Self::DeactivateProfile),
            _ if matches_text_key_in_any_language(text, TextKey::ViewProfiles) => {
                Some(Self::ViewProfiles)
            }
            _ if matches_text_key_in_any_language(text, TextKey::MyProfile) => {
                Some(Self::MyProfile)
            }
            _ if matches_text_key_in_any_language(text, TextKey::Settings) => Some(Self::Settings),
            _ if matches_text_key_in_any_language(text, TextKey::DeactivateProfile) => {
                Some(Self::DeactivateProfile)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDecision {
    Like,
    Skip,
}

impl SwipeDecision {
    pub fn parse(text: &str) -> Option<Self> {
        if matches_text_key_in_any_language(text, TextKey::Like) {
            Some(Self::Like)
        } else if matches_text_key_in_any_language(text, TextKey::Skip) {
            Some(Self::Skip)
        } else {
            None
        }
    }
}

fn matches_text_key_in_any_language(text: &str, key: TextKey) -> bool {
    let text = text.trim();

    [Language::En, Language::Sk, Language::Uk]
        .into_iter()
        .any(|language| text == language.text(key))
}

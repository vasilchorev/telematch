use crate::bot::i18n::TextKey;
use crate::bot::keyboards::{
    EN_INCOMING_LIKE_SHOW_BUTTON_TEXT, EN_INCOMING_LIKE_STOP_BUTTON_TEXT,
    ENGLISH_LANGUAGE_BUTTON_TEXT, SK_INCOMING_LIKE_SHOW_BUTTON_TEXT,
    SK_INCOMING_LIKE_STOP_BUTTON_TEXT, SLOVAK_LANGUAGE_BUTTON_TEXT,
    UK_INCOMING_LIKE_SHOW_BUTTON_TEXT, UK_INCOMING_LIKE_STOP_BUTTON_TEXT,
    UKRAINIAN_LANGUAGE_BUTTON_TEXT,
};
use crate::models::Profile;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub const LANGUAGE_PROMPT: &str = "Choose language / Vyber jazyk / Обери мову";
pub const REVEAL_SPINNER_TEXT: &str = "✨🔍";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Sk,
    Uk,
}

impl Lang {
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

pub fn profile_lang(profile: &Profile) -> Lang {
    profile
        .language_code
        .as_deref()
        .map(Lang::from_db_code)
        .unwrap_or(Lang::En)
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
    AwaitingIncomingLikeDecision {
        profile: Profile,
        liked_you_user_id: i64,
    },
    AwaitingProfileAction {
        profile: Profile,
        shown_profile_user_id: i64,
        return_to_menu: bool,
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
        if matches_text_key_in_any_lang(text, TextKey::SaveProfile) {
            Some(Self::SaveProfile)
        } else if matches_text_key_in_any_lang(text, TextKey::EditProfile) {
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
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuAction {
    ViewProfiles,
    MyProfile,
    DeactivateProfile,
}

impl MainMenuAction {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "1" => Some(Self::ViewProfiles),
            "2" => Some(Self::MyProfile),
            "3" => Some(Self::DeactivateProfile),
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
        if matches_text_key_in_any_lang(text, TextKey::Like) {
            Some(Self::Like)
        } else if matches_text_key_in_any_lang(text, TextKey::Skip) {
            Some(Self::Skip)
        } else {
            None
        }
    }
}

fn matches_text_key_in_any_lang(text: &str, key: TextKey) -> bool {
    let text = text.trim();

    [Lang::En, Lang::Sk, Lang::Uk]
        .into_iter()
        .any(|lang| text == lang.text(key))
}

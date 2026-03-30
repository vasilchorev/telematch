use crate::Lang;
use crate::bot::i18n::TextKey;
use teloxide::types::{ButtonRequest, KeyboardButton, KeyboardMarkup};

pub const ENGLISH_LANGUAGE_BUTTON_TEXT: &str = "🇬🇧 English";
pub const SLOVAK_LANGUAGE_BUTTON_TEXT: &str = "🇸🇰 Slovenčina";
pub const UKRAINIAN_LANGUAGE_BUTTON_TEXT: &str = "🇺🇦 Українська";

pub const EN_INCOMING_LIKE_SHOW_BUTTON_TEXT: &str = "👍 Show";
pub const EN_INCOMING_LIKE_STOP_BUTTON_TEXT: &str = "👎 Stop";

pub const SK_INCOMING_LIKE_SHOW_BUTTON_TEXT: &str = "👍 Zobraziť";
pub const SK_INCOMING_LIKE_STOP_BUTTON_TEXT: &str = "👎 Nechcem ďalej pozerať";

pub const UK_INCOMING_LIKE_SHOW_BUTTON_TEXT: &str = "👍 Показати";
pub const UK_INCOMING_LIKE_STOP_BUTTON_TEXT: &str = "👎 Я більше не хочу нікого дивитись";

pub fn make_main_menu_keyboard(lang: Lang) -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(lang.text(TextKey::ViewProfiles)),
            KeyboardButton::new(lang.text(TextKey::MyProfile)),
        ],
        vec![
            KeyboardButton::new(lang.text(TextKey::DeactivateProfile)),
            KeyboardButton::new(lang.text(TextKey::Settings)),
        ],
    ])
    .resize_keyboard()
}

pub fn make_gender_keyboard(lang: Lang) -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new(lang.text(TextKey::Male)),
        KeyboardButton::new(lang.text(TextKey::Female)),
    ]])
    .resize_keyboard()
    .one_time_keyboard()
}

pub fn make_profile_confirmation_keyboard(lang: Lang) -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new(lang.text(TextKey::SaveProfile)),
        KeyboardButton::new(lang.text(TextKey::EditProfile)),
    ]])
    .resize_keyboard()
    .one_time_keyboard()
}

pub fn make_edit_profile_keyboard(lang: Lang) -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new(lang.text(TextKey::EditProfileMenuAction)),
            KeyboardButton::new(lang.text(TextKey::ChangePhoto)),
        ],
        vec![
            KeyboardButton::new(lang.text(TextKey::ChangeBio)),
            KeyboardButton::new(lang.text(TextKey::BackToMainMenu)),
        ],
    ])
    .resize_keyboard()
}

pub fn make_settings_keyboard(lang: Lang) -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(lang.text(TextKey::ChangeLanguage))],
        vec![KeyboardButton::new(lang.text(TextKey::BackToMainMenu))],
    ])
    .resize_keyboard()
}

pub fn make_profile_action_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new("❤️"),
        KeyboardButton::new("👎"),
    ]])
    .resize_keyboard()
    .one_time_keyboard()
}

pub fn make_language_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new(ENGLISH_LANGUAGE_BUTTON_TEXT),
        KeyboardButton::new(SLOVAK_LANGUAGE_BUTTON_TEXT),
        KeyboardButton::new(UKRAINIAN_LANGUAGE_BUTTON_TEXT),
    ]])
    .resize_keyboard()
    .one_time_keyboard()
}

pub fn make_incoming_like_keyboard(lang: Lang) -> KeyboardMarkup {
    let (show_text, stop_text) = match lang {
        Lang::En => (
            EN_INCOMING_LIKE_SHOW_BUTTON_TEXT,
            EN_INCOMING_LIKE_STOP_BUTTON_TEXT,
        ),
        Lang::Sk => (
            SK_INCOMING_LIKE_SHOW_BUTTON_TEXT,
            SK_INCOMING_LIKE_STOP_BUTTON_TEXT,
        ),
        Lang::Uk => (
            UK_INCOMING_LIKE_SHOW_BUTTON_TEXT,
            UK_INCOMING_LIKE_STOP_BUTTON_TEXT,
        ),
    };

    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new(show_text)],
        vec![KeyboardButton::new(stop_text)],
    ])
    .resize_keyboard()
}

pub fn make_location_choice_keyboard(count: usize) -> KeyboardMarkup {
    let row = (1..=count)
        .map(|i| KeyboardButton::new(i.to_string()))
        .collect::<Vec<_>>();

    KeyboardMarkup::new(vec![row]).resize_keyboard().one_time_keyboard()
}

pub fn make_previous_value_keyboard(text: impl Into<String>) -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![KeyboardButton::new(text)]])
        .resize_keyboard()
        .one_time_keyboard()
}

pub fn make_skip_keyboard(lang: Lang) -> KeyboardMarkup {
    make_previous_value_keyboard(lang.text(TextKey::SkipInput))
}

pub fn make_keep_previous_photo_keyboard(lang: Lang) -> KeyboardMarkup {
    make_previous_value_keyboard(lang.text(TextKey::KeepPreviousPhoto))
}

pub fn make_location_keyboard(lang: Lang, previous_location: Option<&str>) -> KeyboardMarkup {
    let mut rows = Vec::new();

    if let Some(location) = previous_location.map(str::trim).filter(|value| !value.is_empty()) {
        rows.push(vec![KeyboardButton::new(location.to_owned())]);
    }

    rows.push(vec![
        KeyboardButton::new(lang.text(TextKey::SendLocationButton))
            .request(ButtonRequest::Location),
    ]);

    KeyboardMarkup::new(rows)
        .resize_keyboard()
        .one_time_keyboard()
}

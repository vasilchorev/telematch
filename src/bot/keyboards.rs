use crate::Lang;
use crate::bot::i18n::TextKey;
use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub const ENGLISH_LANGUAGE_BUTTON_TEXT: &str = "🇬🇧 English";
pub const SLOVAK_LANGUAGE_BUTTON_TEXT: &str = "🇸🇰 Slovenčina";
pub const UKRAINIAN_LANGUAGE_BUTTON_TEXT: &str = "🇺🇦 Українська";

pub const EN_INCOMING_LIKE_SHOW_BUTTON_TEXT: &str = "👍 Show";
pub const EN_INCOMING_LIKE_STOP_BUTTON_TEXT: &str = "👎 Stop";

pub const SK_INCOMING_LIKE_SHOW_BUTTON_TEXT: &str = "👍 Zobraziť";
pub const SK_INCOMING_LIKE_STOP_BUTTON_TEXT: &str = "👎 Nechcem ďalej pozerať";

pub const UK_INCOMING_LIKE_SHOW_BUTTON_TEXT: &str = "👍 Показати";
pub const UK_INCOMING_LIKE_STOP_BUTTON_TEXT: &str = "👎 Я більше не хочу нікого дивитись";

pub fn make_main_menu_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new("1"),
        KeyboardButton::new("2"),
        KeyboardButton::new("3"),
    ]])
    .resize_keyboard()
    .one_time_keyboard()
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

pub fn make_edit_profile_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new("1"),
        KeyboardButton::new("2"),
        KeyboardButton::new("3"),
        KeyboardButton::new("4"),
    ]])
    .resize_keyboard()
    .one_time_keyboard()
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

use teloxide::types::{KeyboardButton, KeyboardMarkup};
use crate::bot::i18n::TextKey;
use crate::Lang;

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

pub fn see_who_liked_you_keyboard(lang: Lang) -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new(lang.text(TextKey::SeeWhoLikedYou))
    ]])
        .resize_keyboard()
        .one_time_keyboard()
}

pub fn make_language_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new("English"),
        KeyboardButton::new("Slovenčina"),
        KeyboardButton::new("Українська"),
    ]])
        .resize_keyboard()
        .one_time_keyboard()
}
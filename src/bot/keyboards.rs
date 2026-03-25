use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub fn make_start_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("Vytvoriť profil")
        ]
    ]).resize_keyboard().one_time_keyboard()
}

pub fn make_gender_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Muž"), KeyboardButton::new("Žena")],
    ])
        .resize_keyboard()
        .one_time_keyboard()
}

pub fn make_profile_confirmation_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Yes"), KeyboardButton::new("Edit my profile")],
    ]).resize_keyboard().one_time_keyboard()
}

pub fn make_edit_profile_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("1"),
            KeyboardButton::new("2"),
            KeyboardButton::new("3"),
            KeyboardButton::new("4"),
        ]
    ])
        .resize_keyboard().one_time_keyboard()
}

pub fn make_profile_action_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![vec![
        KeyboardButton::new("❤️"),
        KeyboardButton::new("👎"),
    ]])
        .resize_keyboard().one_time_keyboard()
}
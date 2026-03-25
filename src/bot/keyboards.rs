use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub fn make_start_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![
            KeyboardButton::new("Vytvoriť profil")
        ]
    ]).resize_keyboard()
}

pub fn make_gender_keyboard() -> KeyboardMarkup {
    KeyboardMarkup::new(vec![
        vec![KeyboardButton::new("Muž"), KeyboardButton::new("Žena")],
    ])
        .resize_keyboard()
        .one_time_keyboard()
}



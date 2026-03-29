use crate::Lang;

#[derive(Debug, Clone, Copy)]
pub enum TextKey {
    Welcome,
    CreateProfile,
    RebuildProfile,
    WhatIsYourName,
    SelectGender,
    LookingFor,
    AskAge,
    AskLocation,
    AskBio,
    AskPhoto,
    SaveThisProfile,
    UseKeyboardToConfirm,
    SaveProfile,
    EditProfile,
    NonEmptyName,
    NonEmptyBio,
    SendPhotoMessage,
    MainMenuText,
    EditMenuText,
    ViewProfiles,
    MyProfile,
    DeactivateProfile,
    BackToMainMenu,
    ChangePhoto,
    ChangeBio,
    SomeoneLikedYouShowQuestion,
    LikeOrSkip,
    NoNewLikes,
    NoMoreProfiles,
    InactiveNow,
    ThisPersonLikedYou,
    MatchStartChatting,
    MatchNoUsername,
    Male,
    Female,
    Like,
    Skip,
}

impl Lang {
    pub fn text(self, key: TextKey) -> &'static str {
        match self {
            Lang::En => match key {
                TextKey::Welcome => {
                    "Welcome to TeleMatch.\nCreate a profile to start browsing other people."
                }
                TextKey::CreateProfile => "Create profile",
                TextKey::RebuildProfile => "Let's rebuild your profile. What is your name?",
                TextKey::WhatIsYourName => "What is your name?",
                TextKey::SelectGender => "Select your gender.",
                TextKey::LookingFor => "Who are you looking for?",
                TextKey::AskAge => "How old are you?",
                TextKey::AskLocation => "What city are you from?",
                TextKey::AskBio => "Tell me more about yourself.",
                TextKey::AskPhoto => "Send your profile photo.",
                TextKey::SaveThisProfile => "Save this profile?",
                TextKey::UseKeyboardToConfirm => {
                    "Use the keyboard below to confirm or keep editing."
                }
                TextKey::SaveProfile => "Save profile",
                TextKey::EditProfile => "No, edit profile",
                TextKey::NonEmptyName => "Please send a non-empty name.",
                TextKey::NonEmptyBio => "Please send a non-empty bio.",
                TextKey::SendPhotoMessage => "Send a photo message, not a file.",
                TextKey::MainMenuText => {
                    "1. View profiles\n2. My profile\n3. I don't want to show my profile"
                }
                TextKey::EditMenuText => {
                    "1. Edit profile\n2. Change photo\n3. Change bio\n4. Back to main menu"
                }
                TextKey::ViewProfiles => "View profiles",
                TextKey::MyProfile => "My profile",
                TextKey::DeactivateProfile => "I don't want to show my profile",
                TextKey::BackToMainMenu => "Back to main menu",
                TextKey::ChangePhoto => "Change photo",
                TextKey::ChangeBio => "Change bio",
                TextKey::SomeoneLikedYouShowQuestion => {
                    "1 girl liked you, show her?\n\n1. Show.\n2. I don't want to view anyone anymore."
                }
                TextKey::LikeOrSkip => "Use the keyboard to Like or Skip.",
                TextKey::NoNewLikes => "No new likes right now.",
                TextKey::NoMoreProfiles => "No more matching profiles found right now.",
                TextKey::InactiveNow => {
                    "Your profile is now inactive. Other people will no longer see it."
                }
                TextKey::ThisPersonLikedYou => "Someone liked your profile:",
                TextKey::MatchStartChatting => {
                    "Awesome! Hope you have a great time 🙌\n\nStart chatting 👉 "
                }
                TextKey::MatchNoUsername => {
                    "This user does not have a public username, so Telegram may not let you open a direct chat from the bot."
                }
                TextKey::Male => "Male",
                TextKey::Female => "Female",
                TextKey::Like => "❤️",
                TextKey::Skip => "👎",
            },
            Lang::Sk => match key {
                TextKey::Welcome => {
                    "Vitaj v TeleMatch.\nVytvor si profil a začni prezerať ďalších ľudí."
                }
                TextKey::CreateProfile => "Vytvoriť profil",
                TextKey::RebuildProfile => "Poďme si znovu vytvoriť tvoj profil. Ako sa voláš?",
                TextKey::WhatIsYourName => "Ako sa voláš?",
                TextKey::SelectGender => "Vyber svoje pohlavie.",
                TextKey::LookingFor => "Koho hľadáš?",
                TextKey::AskAge => "Koľko máš rokov?",
                TextKey::AskLocation => "Z akého si mesta?",
                TextKey::AskBio => "Napíš o sebe niečo viac.",
                TextKey::AskPhoto => "Pošli svoju profilovú fotku.",
                TextKey::SaveThisProfile => "Uložiť tento profil?",
                TextKey::UseKeyboardToConfirm => {
                    "Pomocou klávesnice nižšie potvrďte alebo pokračujte v úpravách."
                }
                TextKey::SaveProfile => "Uložiť profil",
                TextKey::EditProfile => "Nie, upraviť profil",
                TextKey::NonEmptyName => "Pošli neprázdne meno.",
                TextKey::NonEmptyBio => "Pošli neprázdny popis.",
                TextKey::SendPhotoMessage => "Pošli fotku ako správu, nie ako súbor.",
                TextKey::MainMenuText => {
                    "1. Prezerať profily\n2. Môj profil\n3. Nechcem zobrazovať svoj profil"
                }
                TextKey::EditMenuText => {
                    "1. Upraviť profil\n2. Zmeniť fotku\n3. Zmeniť popis\n4. Späť do hlavného menu"
                }
                TextKey::ViewProfiles => "Prezerať profily",
                TextKey::MyProfile => "Môj profil",
                TextKey::DeactivateProfile => "Nechcem zobrazovať svoj profil",
                TextKey::BackToMainMenu => "Späť do hlavného menu",
                TextKey::ChangePhoto => "Zmeniť fotku",
                TextKey::ChangeBio => "Zmeniť popis",
                TextKey::SomeoneLikedYouShowQuestion => {
                    "Tvoj profil sa páčil 1 dievčaťu, chceš ju zobraziť?\n\n1. Zobraziť.\n2. Už nechcem nikoho pozerať."
                }
                TextKey::LikeOrSkip => "Použi klávesnicu na lajk alebo preskočenie.",
                TextKey::NoNewLikes => "Momentálne nemáš žiadne nové lajky.",
                TextKey::NoMoreProfiles => "Momentálne sa nenašli ďalšie vhodné profily.",
                TextKey::InactiveNow => "Tvoj profil je teraz neaktívny. Ostatní ho už neuvidia.",
                TextKey::ThisPersonLikedYou => "Niekomu sa páčil tvoj profil:",
                TextKey::MatchStartChatting => {
                    "Super! Dúfam, že si spolu užijete čas 🙌\n\nZačni chatovať 👉 "
                }
                TextKey::MatchNoUsername => {
                    "Tento používateľ nemá verejné používateľské meno, takže Telegram nemusí umožniť otvorenie priameho chatu z bota."
                }
                TextKey::Male => "Muž",
                TextKey::Female => "Žena",
                TextKey::Like => "❤️",
                TextKey::Skip => "👎",
            },
            Lang::Uk => match key {
                TextKey::Welcome => {
                    "Ласкаво просимо до TeleMatch.\nСтвори профіль, щоб почати перегляд інших людей."
                }
                TextKey::CreateProfile => "Створити профіль",
                TextKey::RebuildProfile => "Давайте оновимо ваш профіль. Як вас звати?",
                TextKey::WhatIsYourName => "Як тебе звати?",
                TextKey::SelectGender => "Обери свою стать.",
                TextKey::LookingFor => "Кого ти шукаєш?",
                TextKey::AskAge => "Скільки тобі років?",
                TextKey::AskLocation => "З якого ти міста?",
                TextKey::AskBio => "Розкажи трохи більше про себе.",
                TextKey::AskPhoto => "Надішли своє фото профілю.",
                TextKey::SaveThisProfile => "Зберегти цей профіль?",
                TextKey::UseKeyboardToConfirm => {
                    "Використовуйте клавіатуру нижче, щоб підтвердити або продовжити редагування"
                }
                TextKey::SaveProfile => "Зберегти профіль",
                TextKey::EditProfile => "Ні, редагувати профіль",
                TextKey::NonEmptyName => "Надішли непорожнє ім’я.",
                TextKey::NonEmptyBio => "Надішли непорожній опис.",
                TextKey::SendPhotoMessage => "Надішли фото як повідомлення, а не як файл.",
                TextKey::MainMenuText => {
                    "1. Переглядати профілі\n2. Мій профіль\n3. Я не хочу показувати свій профіль"
                }
                TextKey::EditMenuText => {
                    "1. Редагувати профіль\n2. Змінити фото\n3. Змінити опис\n4. Назад у головне меню"
                }
                TextKey::ViewProfiles => "Переглядати профілі",
                TextKey::MyProfile => "Мій профіль",
                TextKey::DeactivateProfile => "Я не хочу показувати свій профіль",
                TextKey::BackToMainMenu => "Назад у головне меню",
                TextKey::ChangePhoto => "Змінити фото",
                TextKey::ChangeBio => "Змінити опис",
                TextKey::SomeoneLikedYouShowQuestion => {
                    "Ти сподобався 1 дівчині, показати її?\n\n1. Показати.\n2. Я більше не хочу нікого дивитись."
                }
                TextKey::LikeOrSkip => "Використай клавіатуру, щоб лайкнути або пропустити.",
                TextKey::NoNewLikes => "Зараз немає нових лайків.",
                TextKey::NoMoreProfiles => "Зараз більше відповідних профілів не знайдено.",
                TextKey::InactiveNow => {
                    "Твій профіль тепер неактивний. Інші його більше не бачитимуть."
                }
                TextKey::ThisPersonLikedYou => "Комусь сподобалась твоя анкета:",
                TextKey::MatchStartChatting => {
                    "Cупер! Сподіваюсь, гарно проведете час 🙌\n\nПочинай спілкуватися 👉 "
                }
                TextKey::MatchNoUsername => {
                    "У цього користувача немає публічного username, тому Telegram може не дозволити відкрити приватний чат із бота."
                }
                TextKey::Male => "Чоловік",
                TextKey::Female => "Жінка",
                TextKey::Like => "❤️",
                TextKey::Skip => "👎",
            },
        }
    }
}

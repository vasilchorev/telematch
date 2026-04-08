use crate::Language;

#[derive(Debug, Clone, Copy)]
pub enum TextKey {
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
    EditProfileMenuAction,
    NonEmptyName,
    NonEmptyBio,
    SendPhotoMessage,
    SkipInput,
    KeepPreviousPhoto,
    MainMenuText,
    SettingsMenuText,
    ViewProfiles,
    MyProfile,
    Settings,
    ChangeLanguage,
    DeactivateProfile,
    BackToMainMenu,
    ChangePhoto,
    ChangeBio,
    IncomingLikePromptFemaleSingular,
    IncomingLikePromptFemalePlural,
    IncomingLikePromptMaleSingular,
    IncomingLikePromptMalePlural,
    IncomingLikePromptNeutralSingular,
    IncomingLikePromptNeutralPlural,
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
    SendLocationButton,
}

impl Language {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn text(self, key: TextKey) -> &'static str {
        match self {
            Self::En => match key {
                TextKey::RebuildProfile => "Let's update your profile. What's your name?",
                TextKey::WhatIsYourName => "What's your name?",
                TextKey::SelectGender => "Select your gender.",
                TextKey::LookingFor => "Who would you like to meet?",
                TextKey::AskAge => "How old are you?",
                TextKey::AskLocation => "Which city are you in?",
                TextKey::AskBio => "Tell us a bit about yourself.",
                TextKey::AskPhoto => "Send your profile photo.",
                TextKey::SaveThisProfile => "Do you want to save this profile?",
                TextKey::UseKeyboardToConfirm => {
                    "Use the keyboard below to save the profile or keep editing."
                }
                TextKey::SaveProfile => "Save profile",
                TextKey::EditProfile => "Keep editing",
                TextKey::EditProfileMenuAction => "Edit profile",
                TextKey::NonEmptyName => "Please enter a name.",
                TextKey::NonEmptyBio => "Please enter a short bio or tap Skip.",
                TextKey::SendPhotoMessage => "Please send the photo as an image, not as a file.",
                TextKey::SkipInput => "Skip",
                TextKey::KeepPreviousPhoto => "Keep current photo",
                TextKey::MainMenuText => "Main menu:",
                TextKey::SettingsMenuText => "Settings:",
                TextKey::ViewProfiles => "Browse profiles",
                TextKey::MyProfile => "My profile",
                TextKey::Settings => "Settings",
                TextKey::ChangeLanguage => "Change language",
                TextKey::DeactivateProfile => "Deactivate profile",
                TextKey::BackToMainMenu => "Back to main menu",
                TextKey::ChangePhoto => "Change photo",
                TextKey::ChangeBio => "Edit bio",
                TextKey::IncomingLikePromptFemaleSingular => {
                    "{count} woman liked your profile. Do you want to see her?\n\n1. Show.\n2. Stop browsing."
                }
                TextKey::IncomingLikePromptFemalePlural => {
                    "{count} women liked your profile. Do you want to see them?\n\n1. Show.\n2. Stop browsing."
                }
                TextKey::IncomingLikePromptMaleSingular => {
                    "{count} man liked your profile. Do you want to see him?\n\n1. Show.\n2. Stop browsing."
                }
                TextKey::IncomingLikePromptMalePlural => {
                    "{count} men liked your profile. Do you want to see them?\n\n1. Show.\n2. Stop browsing."
                }
                TextKey::IncomingLikePromptNeutralSingular => {
                    "{count} person liked your profile. Do you want to see them?\n\n1. Show.\n2. Stop browsing."
                }
                TextKey::IncomingLikePromptNeutralPlural => {
                    "{count} people liked your profile. Do you want to see them?\n\n1. Show.\n2. Stop browsing."
                }
                TextKey::LikeOrSkip => "Use the keyboard to like or skip.",
                TextKey::NoNewLikes => "No new likes yet.",
                TextKey::NoMoreProfiles => "There are no more matching profiles right now.",
                TextKey::InactiveNow => {
                    "Your profile is now inactive. Other people won't see it anymore."
                }
                TextKey::ThisPersonLikedYou => "This person liked your profile:",
                TextKey::MatchStartChatting => "It's a match!\n\nStart chatting 👉 ",
                TextKey::MatchNoUsername => {
                    "This user doesn't have a public username, so Telegram may not let you open a direct chat from the bot."
                }
                TextKey::Male => "Male",
                TextKey::Female => "Female",
                TextKey::Like => "❤️",
                TextKey::Skip => "👎",
                TextKey::SendLocationButton => "📍 Share location",
            },
            Self::Sk => match key {
                TextKey::RebuildProfile => "Poďme upraviť tvoj profil. Ako sa voláš?",
                TextKey::WhatIsYourName => "Ako sa voláš?",
                TextKey::SelectGender => "Vyber svoje pohlavie.",
                TextKey::LookingFor => "Koho chceš spoznať?",
                TextKey::AskAge => "Koľko máš rokov?",
                TextKey::AskLocation => "V ktorom meste bývaš?",
                TextKey::AskBio => "Napíš o sebe pár viet.",
                TextKey::AskPhoto => "Pošli svoju profilovú fotku.",
                TextKey::SaveThisProfile => "Chceš si uložiť tento profil?",
                TextKey::UseKeyboardToConfirm => {
                    "Pomocou klávesnice nižšie profil ulož alebo pokračuj v úpravách."
                }
                TextKey::SaveProfile => "Uložiť profil",
                TextKey::EditProfile => "Pokračovať v úpravách",
                TextKey::EditProfileMenuAction => "Upraviť profil",
                TextKey::NonEmptyName => "Prosím, zadaj meno.",
                TextKey::NonEmptyBio => "Prosím, napíš krátky popis alebo stlač Preskočiť.",
                TextKey::SendPhotoMessage => "Pošli fotku ako obrázok, nie ako súbor.",
                TextKey::SkipInput => "Preskočiť",
                TextKey::KeepPreviousPhoto => "Ponechať aktuálnu fotku",
                TextKey::MainMenuText => "Menu:",
                TextKey::SettingsMenuText => "Nastavenia:",
                TextKey::ViewProfiles => "Prezerať profily",
                TextKey::MyProfile => "Môj profil",
                TextKey::Settings => "Nastavenia",
                TextKey::ChangeLanguage => "Zmeniť jazyk",
                TextKey::DeactivateProfile => "Deaktivovať profil",
                TextKey::BackToMainMenu => "Späť do hlavného menu",
                TextKey::ChangePhoto => "Zmeniť fotku",
                TextKey::ChangeBio => "Upraviť popis",
                TextKey::IncomingLikePromptFemaleSingular => {
                    "Tvoj profil sa páčil {count} žene. Chceš si ju pozrieť?\n\n1. Zobraziť.\n2. Ukončiť prezeranie."
                }
                TextKey::IncomingLikePromptFemalePlural => {
                    "Tvoj profil sa páčil {count} ženám. Chceš si ich pozrieť?\n\n1. Zobraziť.\n2. Ukončiť prezeranie."
                }
                TextKey::IncomingLikePromptMaleSingular => {
                    "Tvoj profil sa páčil {count} mužovi. Chceš si ho pozrieť?\n\n1. Zobraziť.\n2. Ukončiť prezeranie."
                }
                TextKey::IncomingLikePromptMalePlural => {
                    "Tvoj profil sa páčil {count} mužom. Chceš si ich pozrieť?\n\n1. Zobraziť.\n2. Ukončiť prezeranie."
                }
                TextKey::IncomingLikePromptNeutralSingular => {
                    "Tvoj profil sa páčil {count} človeku. Chceš si ho pozrieť?\n\n1. Zobraziť.\n2. Ukončiť prezeranie."
                }
                TextKey::IncomingLikePromptNeutralPlural => {
                    "Tvoj profil sa páčil {count} ľuďom. Chceš si ich pozrieť?\n\n1. Zobraziť.\n2. Ukončiť prezeranie."
                }
                TextKey::LikeOrSkip => "Použi klávesnicu na lajk alebo preskočenie.",
                TextKey::NoNewLikes => "Zatiaľ nemáš žiadne nové lajky.",
                TextKey::NoMoreProfiles => "Momentálne pre teba nemáme ďalšie profily.",
                TextKey::InactiveNow => "Tvoj profil je teraz neaktívny. Ostatní ho už neuvidia.",
                TextKey::ThisPersonLikedYou => "Tomuto človeku sa páčil tvoj profil:",
                TextKey::MatchStartChatting => "Je to zhoda!\n\nZačni chatovať 👉 ",
                TextKey::MatchNoUsername => {
                    "Tento používateľ nemá verejné používateľské meno, takže Telegram nemusí umožniť otvorenie priameho chatu z bota."
                }
                TextKey::Male => "Muž",
                TextKey::Female => "Žena",
                TextKey::Like => "❤️",
                TextKey::Skip => "👎",
                TextKey::SendLocationButton => "📍 Zdieľať polohu",
            },
            Self::Uk => match key {
                TextKey::RebuildProfile => "Давай оновимо твій профіль. Як тебе звати?",
                TextKey::WhatIsYourName => "Як тебе звати?",
                TextKey::SelectGender => "Обери свою стать.",
                TextKey::LookingFor => "Кого ти хочеш знайти?",
                TextKey::AskAge => "Скільки тобі років?",
                TextKey::AskLocation => "У якому місті ти живеш?",
                TextKey::AskBio => "Розкажи трохи про себе.",
                TextKey::AskPhoto => "Надішли фото профілю.",
                TextKey::SaveThisProfile => "Зберегти цей профіль?",
                TextKey::UseKeyboardToConfirm => {
                    "Скористайся клавіатурою нижче, щоб зберегти профіль або продовжити редагування."
                }
                TextKey::SaveProfile => "Зберегти профіль",
                TextKey::EditProfile => "Продовжити редагування",
                TextKey::EditProfileMenuAction => "Редагувати профіль",
                TextKey::NonEmptyName => "Будь ласка, напиши ім'я.",
                TextKey::NonEmptyBio => {
                    "Будь ласка, напиши короткий опис або натисни «Пропустити»."
                }
                TextKey::SendPhotoMessage => "Надішли фото як зображення, а не як файл.",
                TextKey::SkipInput => "Пропустити",
                TextKey::KeepPreviousPhoto => "Залишити поточне фото",
                TextKey::MainMenuText => "Меню:",
                TextKey::SettingsMenuText => "Налаштування:",
                TextKey::ViewProfiles => "Переглядати анкети",
                TextKey::MyProfile => "Мій профіль",
                TextKey::Settings => "Налаштування",
                TextKey::ChangeLanguage => "Змінити мову",
                TextKey::DeactivateProfile => "Деактивувати профіль",
                TextKey::BackToMainMenu => "Назад у головне меню",
                TextKey::ChangePhoto => "Змінити фото",
                TextKey::ChangeBio => "Змінити опис",
                TextKey::IncomingLikePromptFemaleSingular => {
                    "Твій профіль сподобався {count} дівчині. Показати її?\n\n1. Показати.\n2. Припинити перегляд."
                }
                TextKey::IncomingLikePromptFemalePlural => {
                    "Твій профіль сподобався {count} дівчатам. Показати їх?\n\n1. Показати.\n2. Припинити перегляд."
                }
                TextKey::IncomingLikePromptMaleSingular => {
                    "Твій профіль сподобався {count} чоловікові. Показати його?\n\n1. Показати.\n2. Припинити перегляд."
                }
                TextKey::IncomingLikePromptMalePlural => {
                    "Твій профіль сподобався {count} чоловікам. Показати їх?\n\n1. Показати.\n2. Припинити перегляд."
                }
                TextKey::IncomingLikePromptNeutralSingular => {
                    "Твій профіль сподобався {count} людині. Показати її?\n\n1. Показати.\n2. Припинити перегляд."
                }
                TextKey::IncomingLikePromptNeutralPlural => {
                    "Твій профіль сподобався {count} людям. Показати їх?\n\n1. Показати.\n2. Припинити перегляд."
                }
                TextKey::LikeOrSkip => "Скористайся клавіатурою, щоб лайкнути або пропустити.",
                TextKey::NoNewLikes => "Поки що нових лайків немає.",
                TextKey::NoMoreProfiles => "Зараз немає нових відповідних профілів.",
                TextKey::InactiveNow => {
                    "Твій профіль тепер неактивний. Інші користувачі більше його не бачитимуть."
                }
                TextKey::ThisPersonLikedYou => "Цей користувач уподобав твій профіль:",
                TextKey::MatchStartChatting => "Це взаємний лайк!\n\nПочинай спілкування 👉 ",
                TextKey::MatchNoUsername => {
                    "У цього користувача немає публічного username, тому Telegram може не дозволити відкрити приватний чат із бота."
                }
                TextKey::Male => "Чоловік",
                TextKey::Female => "Жінка",
                TextKey::Like => "❤️",
                TextKey::Skip => "👎",
                TextKey::SendLocationButton => "📍 Поділитися локацією",
            },
        }
    }
}

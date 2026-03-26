mod bot;
mod db;
mod models;

use crate::bot::i18n::TextKey;
use crate::bot::keyboards::{make_edit_profile_keyboard, make_gender_keyboard, make_profile_action_keyboard, make_profile_confirmation_keyboard, see_who_liked_you_keyboard, make_main_menu_keyboard, make_language_keyboard};
use crate::db::connect_db;
use crate::db::profile_repository::{ProfileRow, get_next_profile_for_user, get_profile_by_user_id, save_profile, activate_profile, deactivate_profile};
use crate::db::swipe_repository::{did_user_like_me, get_incoming_like_for_user, save_swipe};
use crate::models::{CompleteProfile, Gender, Profile};
use sqlx::PgPool;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestLinkPreviewExt;
use teloxide::types::{FileId, InputFile, ParseMode, ReplyMarkup};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Sk,
    Uk,
}

impl Lang {
    pub fn from_text(text: &str) -> Option<Self> {
        match text.trim() {
            "English" => Some(Self::En),
            "Slovenčina" | "Slovencina" => Some(Self::Sk),
            "Українська" => Some(Self::Uk),
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

fn profile_lang(profile: &Profile) -> Lang {
    profile
        .language_code
        .as_deref()
        .map(Lang::from_db_code)
        .unwrap_or(Lang::En)
}

#[derive(Clone, Default)]
enum State {
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
    AwaitingProfileAction {
        profile: Profile,
        shown_profile_user_id: i64,
        return_to_menu: bool,
    },
}

#[derive(Debug, Clone)]
struct SenderInfo {
    telegram_user_id: i64,
    telegram_username: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfirmationAction {
    SaveProfile,
    EditProfile,
}

impl ConfirmationAction {
    fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "Save profile" | "Uložiť profil" | "Зберегти профіль" => Some(Self::SaveProfile),
            "No, edit profile" | "Nie, upraviť profil" | "Ні, редагувати профіль" => {
                Some(Self::EditProfile)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditMenuAction {
    EditProfile,
    ChangePhoto,
    ChangeBio,
    BackToMainMenu,
}

impl EditMenuAction {
    fn parse(text: &str) -> Option<Self> {
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
enum MainMenuAction {
    ViewProfiles,
    MyProfile,
    DeactivateProfile,
}

impl MainMenuAction {
    fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "1" => Some(Self::ViewProfiles),
            "2" => Some(Self::MyProfile),
            "3" => {
                Some(Self::DeactivateProfile)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwipeDecision {
    Like,
    Skip,
}

impl SwipeDecision {
    fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "❤️" => Some(Self::Like),
            "👎" => Some(Self::Skip),
            _ => None,
        }
    }
}

fn is_start_command(msg: Message) -> bool {
    msg.text()
        .map(|text| text.trim_start().starts_with("/start"))
        .unwrap_or(false)
}

fn is_see_who_liked_you_message(msg: Message) -> bool {
    let Some(text) = msg.text() else {
        return false;
    };

    let text = text.trim();

    [
        Lang::En.text(TextKey::SeeWhoLikedYou),
        Lang::Sk.text(TextKey::SeeWhoLikedYou),
        Lang::Uk.text(TextKey::SeeWhoLikedYou),
    ]
        .contains(&text)
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn telegram_username_link_html(username: &str, name: &str) -> String {
    format!(
        r#"<a href="https://t.me/{}">{}</a>"#,
        html_escape(username),
        html_escape(name)
    )
}

fn profile_contact_link(profile_row: &ProfileRow) -> Option<String> {
    profile_row
        .username
        .as_deref()
        .filter(|username| !username.is_empty())
        .map(|username| telegram_username_link_html(username, &profile_row.name))
}

fn match_notification_text(profile_row: &ProfileRow, lang: Lang) -> String {
    match profile_contact_link(profile_row) {
        Some(link) => format!("{}{}", lang.text(TextKey::MatchStartChatting), link),
        None => format!(
            "{} {}",
            lang.text(TextKey::MatchNoUsername),
            html_escape(&profile_row.name)
        ),
    }
}

fn extract_photo_file_id(msg: &Message) -> Option<String> {
    msg.photo()
        .and_then(|photo_sizes| photo_sizes.last())
        .map(|largest_photo| largest_photo.file.id.to_string())
}

async fn require_sender(
    bot: &Bot,
    msg: &Message,
) -> Result<Option<SenderInfo>, teloxide::RequestError> {
    let Some(user) = msg.from.as_ref() else {
        log::warn!("Received a message without a Telegram sender");
        bot.send_message(
            msg.chat.id,
            "I could not identify the sender for this message. Please try again from a private chat.",
        )
        .await?;
        return Ok(None);
    };

    Ok(Some(SenderInfo {
        telegram_user_id: user.id.0 as i64,
        telegram_username: user.username.clone(),
    }))
}

async fn send_profile(
    bot: &Bot,
    chat_id: ChatId,
    profile: &Profile,
    footer_text: Option<&str>,
    reply_markup: Option<ReplyMarkup>,
) -> HandlerResult {
    let mut text = profile.display_text();

    if let Some(footer) = footer_text {
        text.push_str("\n\n");
        text.push_str(footer);
    }

    if let Some(photo) = profile.photo.as_deref() {
        let mut request = bot
            .send_photo(chat_id, InputFile::file_id(FileId::from(photo.to_owned())))
            .caption(text);

        if let Some(markup) = reply_markup {
            request = request.reply_markup(markup);
        }

        request.await?;
    } else {
        let mut request = bot.send_message(chat_id, text);

        if let Some(markup) = reply_markup {
            request = request.reply_markup(markup);
        }

        request.await?;
    }

    Ok(())
}

async fn show_edit_menu(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(
        chat_id,
        lang.text(TextKey::EditMenuText)
    )
        .reply_markup(make_edit_profile_keyboard())
        .await?;

    Ok(())
}

async fn show_main_menu(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, lang.text(TextKey::MainMenuText))
        .reply_markup(make_main_menu_keyboard())
        .await?;

    Ok(())
}

async fn prompt_for_profile_action(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, lang.text(TextKey::LikeOrSkip))
        .reply_markup(make_profile_action_keyboard())
        .await?;

    Ok(())
}

async fn preview_profile_for_confirmation(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
    draft: &Profile,
) -> HandlerResult {
    send_profile(bot, chat_id, draft, None, None).await?;
    bot.send_message(chat_id, lang.text(TextKey::SaveThisProfile))
        .reply_markup(make_profile_confirmation_keyboard(lang))
        .await?;

    Ok(())
}

async fn show_profile_for_action(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    my_profile: Profile,
    shown_profile_user_id: i64,
    shown_profile: &Profile,
    footer_text: Option<&str>,
    return_to_menu: bool,
) -> HandlerResult {
    send_profile(
        bot,
        chat_id,
        shown_profile,
        footer_text,
        Some(make_profile_action_keyboard().into()),
    )
    .await?;

    dialogue
        .update(State::AwaitingProfileAction {
            profile: my_profile,
            shown_profile_user_id,
            return_to_menu,
        })
        .await?;

    Ok(())
}

async fn notify_like_target(bot: &Bot, target_user_id: i64, pool: &PgPool) -> HandlerResult {
    let Some(target_row) = get_profile_by_user_id(pool, target_user_id).await? else {
        log::warn!("Unable to notify liked user {target_user_id}: profile not found");
        return Ok(());
    };

    let Some(chat_id) = target_row.chat_id else {
        log::warn!("Unable to notify liked user {target_user_id}: chat_id is missing");
        return Ok(());
    };


    let lang = Lang::from_db_code(&target_row.language_code);
    bot.send_message(ChatId(chat_id), lang.text(TextKey::SomeoneLikedYou))
        .reply_markup(see_who_liked_you_keyboard(lang))
        .await?;

    Ok(())
}

async fn notify_mutual_match(
    bot: &Bot,
    requester_chat_id: ChatId,
    requester_user_id: i64,
    other_user_id: i64,
    lang:Lang,
    pool: &PgPool,
) -> HandlerResult {
    let Some(requester_row) = get_profile_by_user_id(pool, requester_user_id).await? else {
        log::warn!("Unable to finalize match for user {requester_user_id}: profile not found");
        return Ok(());
    };

    let Some(other_row) = get_profile_by_user_id(pool, other_user_id).await? else {
        log::warn!("Unable to finalize match with user {other_user_id}: profile not found");
        return Ok(());
    };

    bot.send_message(requester_chat_id, match_notification_text(&other_row, lang))
        .parse_mode(ParseMode::Html)
        .disable_link_preview(true)
        .await?;

    let Some(other_chat_id) = other_row.chat_id else {
        log::warn!("Unable to notify matched user {other_user_id}: chat_id is missing");
        return Ok(());
    };

    let other_lang = Lang::from_db_code(&other_row.language_code);
    send_profile(
        bot,
        ChatId(other_chat_id),
        &requester_row.to_profile(),
        None,
        None,
    )
    .await?;
    bot.send_message(
        ChatId(other_chat_id),
        match_notification_text(&requester_row, other_lang),
    )
    .parse_mode(ParseMode::Html)
    .disable_link_preview(true)
    .await?;

    show_main_menu(&bot, ChatId(other_chat_id), other_lang).await?;

    Ok(())
}

async fn show_next_profile_or_menu(
    bot: &Bot,
    dialogue: &MyDialogue,
    msg: &Message,
    my_profile: Profile,
    pool: &PgPool,
) -> HandlerResult {
    let Some(sender) = require_sender(bot, msg).await? else {
        return Ok(());
    };

    let lang = profile_lang(&my_profile);
    if let Some(incoming_like_row) =
        get_incoming_like_for_user(pool, sender.telegram_user_id).await?
    {
        let shown_profile_user_id = incoming_like_row.telegram_user_id;
        let shown_profile = incoming_like_row.to_profile();

        show_profile_for_action(
            bot,
            dialogue,
            msg.chat.id,
            my_profile,
            shown_profile_user_id,
            &shown_profile,
            Some(lang.text(TextKey::ThisPersonAlreadyLikedYou)),
            false,
        )
        .await?;

        return Ok(());
    }

    let Some(next_profile_row) = get_next_profile_for_user(pool, sender.telegram_user_id).await?
    else {
        bot.send_message(msg.chat.id, lang.text(TextKey::NoMoreProfiles))
            .await?;
        show_main_menu(bot, msg.chat.id, lang).await?;
        dialogue
            .update(State::MainMenu {
                profile: my_profile,
            })
            .await?;
        return Ok(());
    };

    let shown_profile_user_id = next_profile_row.telegram_user_id;
    let shown_profile = next_profile_row.to_profile();

    show_profile_for_action(
        bot,
        dialogue,
        msg.chat.id,
        my_profile,
        shown_profile_user_id,
        &shown_profile,
        None,
        false,
    )
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting TeleMatch bot");

    let bot = Bot::from_env();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool: PgPool = connect_db(&database_url)
        .await
        .expect("Failed to connect to database");

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::filter(is_start_command).endpoint(global_start))
            .branch(dptree::filter(is_see_who_liked_you_message).endpoint(global_see_who_liked_you))
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::WaitingForLanguage].endpoint(receive_language))
            .branch(dptree::case![State::WaitingForName { draft }].endpoint(receive_name))            .branch(dptree::case![State::WaitingForGender { draft }].endpoint(receive_gender))
            .branch(
                dptree::case![State::WaitingForLookingFor { draft }].endpoint(receive_looking_for),
            )
            .branch(dptree::case![State::WaitingForAge { draft }].endpoint(receive_age))
            .branch(dptree::case![State::WaitingForLocation { draft }].endpoint(receive_location))
            .branch(
                dptree::case![State::WaitingForDescription { draft }].endpoint(receive_description),
            )
            .branch(dptree::case![State::WaitingForPhoto { draft }].endpoint(receive_photo))
            .branch(dptree::case![State::ConfirmProfile { draft }].endpoint(confirm_profile))
            .branch(dptree::case![State::EditMenu { profile }].endpoint(edit_profile_menu))
            .branch(dptree::case![State::WaitingForPhotoEdit { draft }].endpoint(edit_photo))
            .branch(
                dptree::case![State::WaitingForDescriptionEdit { draft }]
                    .endpoint(edit_description),
            )
            .branch(
                dptree::case![State::AwaitingProfileAction {
                    profile,
                    shown_profile_user_id,
                    return_to_menu
                }]
                .endpoint(handle_profile_action),
            ).branch(dptree::case![State::MainMenu { profile }].endpoint(main_menu)),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn global_start(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
) -> HandlerResult {
    let Some(sender) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    if let Some(profile_row) = get_profile_by_user_id(&pool, sender.telegram_user_id).await? {
        let profile = profile_row.to_profile();
        let lang = profile_lang(&profile);

        send_profile(&bot, msg.chat.id, &profile, None, None).await?;
        show_main_menu(&bot, msg.chat.id, lang).await?;
        dialogue.update(State::MainMenu { profile }).await?;
    } else {
        bot.send_message(msg.chat.id, "Choose language / Vyber jazyk / Обери мову")
            .reply_markup(make_language_keyboard())
            .await?;
        dialogue.update(State::WaitingForLanguage).await?;
    }

    Ok(())
}

async fn global_see_who_liked_you(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
) -> HandlerResult {
    let Some(sender) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    let Some(my_profile_row) = get_profile_by_user_id(&pool, sender.telegram_user_id).await? else {
        bot.send_message(msg.chat.id, "Choose language / Vyber jazyk / Обери мову")
            .reply_markup(make_language_keyboard())
            .await?;
        dialogue.update(State::WaitingForLanguage).await?;
        return Ok(());
    };

    let my_profile = my_profile_row.to_profile();
    let lang = profile_lang(&my_profile);

    let Some(incoming_like_row) =
        get_incoming_like_for_user(&pool, sender.telegram_user_id).await?
    else {
        bot.send_message(msg.chat.id, lang.text(TextKey::NoNewLikes))
            .await?;
        show_main_menu(&bot, msg.chat.id, lang).await?;
        dialogue
            .update(State::MainMenu {
                profile: my_profile,
            })
            .await?;
        return Ok(());
    };

    let shown_profile_user_id = incoming_like_row.telegram_user_id;
    let shown_profile = incoming_like_row.to_profile();

    show_profile_for_action(
        &bot,
        &dialogue,
        msg.chat.id,
        my_profile,
        shown_profile_user_id,
        &shown_profile,
        Some(lang.text(TextKey::ThisPersonLikedYou)),
        true,
    )
        .await?;

    Ok(())
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Choose language / Vyber jazyk / Обери мову")
            .reply_markup(make_language_keyboard())
            .await?;
        dialogue.update(State::WaitingForLanguage).await?;
        return Ok(());
    };

    match text.trim() {
        "/start" => {
            bot.send_message(msg.chat.id, "Choose language / Vyber jazyk / Обери мову")
                .reply_markup(make_language_keyboard())
                .await?;
            dialogue.update(State::WaitingForLanguage).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Choose language / Vyber jazyk / Обери мову")
                .reply_markup(make_language_keyboard())
                .await?;
            dialogue.update(State::WaitingForLanguage).await?;
        }
    }

    Ok(())
}

async fn receive_language(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Choose language / Vyber jazyk / Обери мову")
            .reply_markup(make_language_keyboard())
            .await?;
        return Ok(());
    };

    let Some(lang) = Lang::from_text(text) else {
        bot.send_message(msg.chat.id, "Choose language / Vyber jazyk / Обери мову")
            .reply_markup(make_language_keyboard())
            .await?;
        return Ok(());
    };

    let draft = Profile {
        chat_id: Some(msg.chat.id.0),
        language_code: Some(lang.as_db_code().to_owned()),
        ..Profile::default()
    };

    bot.send_message(msg.chat.id, lang.text(TextKey::WhatIsYourName))
        .await?;
    dialogue.update(State::WaitingForName { draft }).await?;

    Ok(())
}

async fn receive_name(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let Some(name) = msg.text().map(str::trim).filter(|text| !text.is_empty()) else {
        bot.send_message(msg.chat.id, "Please send a non-empty name.")
            .await?;
        return Ok(());
    };

    let Some(sender) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    draft.telegram_user_id = Some(sender.telegram_user_id);
    draft.chat_id = Some(msg.chat.id.0);
    draft.username = sender.telegram_username;
    draft.name = Some(name.to_owned());

    let lang = profile_lang(&draft);

    bot.send_message(msg.chat.id, lang.text(TextKey::SelectGender))
        .reply_markup(make_gender_keyboard(lang))
        .await?;

    dialogue.update(State::WaitingForGender { draft }).await?;
    Ok(())
}

async fn receive_gender(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, lang.text(TextKey::SelectGender))
            .reply_markup(make_gender_keyboard(lang))
            .await?;
        return Ok(());
    };

    let Some(gender) = Gender::from_text(text, lang) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::SelectGender))
            .reply_markup(make_gender_keyboard(lang))
            .await?;
        return Ok(());
    };

    draft.gender = Some(gender);

    bot.send_message(msg.chat.id, lang.text(TextKey::LookingFor))
        .reply_markup(make_gender_keyboard(lang))
        .await?;

    dialogue.update(State::WaitingForLookingFor { draft }).await?;
    Ok(())
}

async fn receive_looking_for(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, lang.text(TextKey::LookingFor))
            .reply_markup(make_gender_keyboard(lang))
            .await?;
        return Ok(());
    };

    let Some(looking_for) = Gender::from_text(text, lang) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::LookingFor))
            .reply_markup(make_gender_keyboard(lang))
            .await?;
        return Ok(());
    };

    draft.looking_for = Some(looking_for);

    bot.send_message(msg.chat.id, lang.text(TextKey::AskAge)).await?;
    dialogue.update(State::WaitingForAge { draft }).await?;
    Ok(())
}

async fn receive_age(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(text) = msg.text().map(str::trim) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::AskAge))
            .await?;
        return Ok(());
    };

    let age = match text.parse::<u8>() {
        Ok(age) if (1..=100).contains(&age) => age,
        _ => {
            bot.send_message(msg.chat.id, lang.text(TextKey::AskAge))
                .await?;
            return Ok(());
        }
    };

    draft.age = Some(age);

    bot.send_message(msg.chat.id, lang.text(TextKey::AskLocation))
        .await?;

    dialogue.update(State::WaitingForLocation { draft }).await?;
    Ok(())
}

async fn receive_location(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(location) = msg.text().map(str::trim).filter(|text| !text.is_empty()) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::AskLocation))
            .await?;
        return Ok(());
    };

    draft.location = Some(location.to_owned());

    bot.send_message(msg.chat.id, lang.text(TextKey::AskBio))
        .await?;

    dialogue
        .update(State::WaitingForDescription { draft })
        .await?;
    Ok(())
}

async fn receive_description(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(description) = msg.text().map(str::trim).filter(|text| !text.is_empty()) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::AskBio))
            .await?;
        return Ok(());
    };

    draft.description = Some(description.to_owned());

    bot.send_message(msg.chat.id, lang.text(TextKey::AskPhoto))
        .await?;

    dialogue.update(State::WaitingForPhoto { draft }).await?;
    Ok(())
}

async fn receive_photo(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(photo_file_id) = extract_photo_file_id(&msg) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::AskPhoto))
            .await?;
        return Ok(());
    };

    draft.photo = Some(photo_file_id);

    preview_profile_for_confirmation(&bot, msg.chat.id,lang, &draft ).await?;
    dialogue.update(State::ConfirmProfile { draft }).await?;
    Ok(())
}

async fn confirm_profile(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
    pool: PgPool,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(text) = msg.text() else {
        bot.send_message(
            msg.chat.id,
            lang.text(TextKey::UseKeyboardToConfirm),
        )
            .reply_markup(make_profile_confirmation_keyboard(lang))
            .await?;
        return Ok(());
    };

    match ConfirmationAction::parse(text) {
        Some(ConfirmationAction::SaveProfile) => {
            let Some(sender) = require_sender(&bot, &msg).await? else {
                return Ok(());
            };

            draft.telegram_user_id = Some(sender.telegram_user_id);
            draft.chat_id = Some(msg.chat.id.0);
            draft.username = sender.telegram_username;

            let complete_profile = match CompleteProfile::try_from(&draft) {
                Ok(profile) => profile,
                Err(error) => {
                    log::warn!(
                        "Refusing to save incomplete profile for user {}: {}",
                        sender.telegram_user_id,
                        error
                    );
                    bot.send_message(
                        msg.chat.id,
                        "The profile is incomplete. Please edit it again from the beginning.",
                    )
                        .await?;
                    dialogue.update(State::WaitingForName { draft }).await?;
                    return Ok(());
                }
            };

            save_profile(&pool, &complete_profile).await?;

            log::info!("Saved profile for user {}", sender.telegram_user_id);
            show_main_menu(&bot, msg.chat.id, lang).await?;
            dialogue.update(State::MainMenu { profile: draft }).await?;
        }
        Some(ConfirmationAction::EditProfile) => {
            show_edit_menu(&bot, msg.chat.id, lang).await?;
            dialogue.update(State::EditMenu { profile: draft }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Choose one of the available actions.")
                .reply_markup(make_profile_confirmation_keyboard(lang))
                .await?;
        }
    }

    Ok(())
}

async fn edit_profile_menu(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    profile: Profile,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    let Some(text) = msg.text() else {
        show_edit_menu(&bot, msg.chat.id, lang).await?;
        return Ok(());
    };

    match EditMenuAction::parse(text) {
        Some(EditMenuAction::EditProfile) => {
            bot.send_message(
                msg.chat.id,
                lang.text(TextKey::RebuildProfile),
            )
            .await?;
            dialogue.update(State::WaitingForName{ draft: profile}).await?;
        }
        Some(EditMenuAction::ChangePhoto) => {
            bot.send_message(msg.chat.id, lang.text(TextKey::AskPhoto))
                .await?;
            dialogue
                .update(State::WaitingForPhotoEdit { draft: profile })
                .await?;
        }
        Some(EditMenuAction::ChangeBio) => {
            bot.send_message(msg.chat.id, lang.text(TextKey::AskBio)).await?;
            dialogue
                .update(State::WaitingForDescriptionEdit { draft: profile })
                .await?;
        }
        Some(EditMenuAction::BackToMainMenu) => {
            show_main_menu(&bot, msg.chat.id, lang).await?;
            dialogue.update(State::MainMenu { profile }).await?;
        }
        None => {
            show_edit_menu(&bot, msg.chat.id, lang).await?;
        }
    }

    Ok(())
}

async fn edit_photo(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);
    let Some(photo_file_id) = extract_photo_file_id(&msg) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::SendPhotoMessage))
            .await?;
        return Ok(());
    };

    draft.photo = Some(photo_file_id);

    preview_profile_for_confirmation(&bot, msg.chat.id, lang, &draft).await?;
    dialogue.update(State::ConfirmProfile { draft }).await?;
    Ok(())
}

async fn edit_description(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(description) = msg.text().map(str::trim).filter(|text| !text.is_empty()) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::NonEmptyBio))
            .await?;
        return Ok(());
    };

    draft.description = Some(description.to_owned());

    preview_profile_for_confirmation(&bot, msg.chat.id,lang, &draft).await?;
    dialogue.update(State::ConfirmProfile { draft }).await?;
    Ok(())
}

async fn handle_profile_action(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (profile, shown_profile_user_id, return_to_menu): (Profile, i64, bool),
    pool: PgPool,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    let Some(text) = msg.text() else {
        prompt_for_profile_action(&bot, msg.chat.id,lang).await?;
        return Ok(());
    };

    let Some(decision) = SwipeDecision::parse(text) else {
        prompt_for_profile_action(&bot, msg.chat.id, lang).await?;
        return Ok(());
    };

    let Some(sender) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    match decision {
        SwipeDecision::Like => {
            log::info!(
                "User {} liked user {}",
                sender.telegram_user_id,
                shown_profile_user_id
            );

            save_swipe(&pool, sender.telegram_user_id, shown_profile_user_id, true).await?;

            if did_user_like_me(&pool, sender.telegram_user_id, shown_profile_user_id).await? {
                notify_mutual_match(
                    &bot,
                    msg.chat.id,
                    sender.telegram_user_id,
                    shown_profile_user_id,
                    lang,
                    &pool,
                )
                .await?;
            } else {
                notify_like_target(&bot, shown_profile_user_id, &pool).await?;
            }
        }
        SwipeDecision::Skip => {
            log::info!(
                "User {} skipped user {}",
                sender.telegram_user_id,
                shown_profile_user_id
            );

            save_swipe(&pool, sender.telegram_user_id, shown_profile_user_id, false).await?;
        }
    }

    if return_to_menu {
        show_main_menu(&bot, msg.chat.id, lang).await?;
        dialogue.update(State::MainMenu { profile }).await?;
    } else {
        show_next_profile_or_menu(&bot, &dialogue, &msg, profile, &pool).await?;
    }

    Ok(())
}

async fn main_menu(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    profile: Profile,
    pool: PgPool,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    let Some(text) = msg.text() else {
        show_main_menu(&bot, msg.chat.id,lang).await?;
        return Ok(());
    };

    match MainMenuAction::parse(text) {
        Some(MainMenuAction::ViewProfiles) => {
            let Some(sender) = require_sender(&bot, &msg).await? else {
                return Ok(());
            };

            activate_profile(&pool, sender.telegram_user_id).await?;
            show_next_profile_or_menu(&bot, &dialogue, &msg, profile, &pool).await?;
        }
        Some(MainMenuAction::MyProfile) => {
            send_profile(&bot, msg.chat.id, &profile, None, None).await?;
            show_edit_menu(&bot, msg.chat.id, lang).await?;
            dialogue.update(State::EditMenu { profile }).await?;
        }
        Some(MainMenuAction::DeactivateProfile) => {
            let Some(sender) = require_sender(&bot, &msg).await? else {
                return Ok(());
            };

            deactivate_profile(&pool, sender.telegram_user_id).await?;

            bot.send_message(
                msg.chat.id,
                lang.text(TextKey::InactiveNow),
            )
                .await?;

            show_main_menu(&bot, msg.chat.id, lang).await?;
            dialogue.update(State::MainMenu { profile }).await?;
        }
        None => {
            show_main_menu(&bot, msg.chat.id, lang).await?;
        }
    }

    Ok(())
}
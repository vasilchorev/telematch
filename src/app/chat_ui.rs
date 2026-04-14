use crate::app::types::{
    AppDialogue, HandlerResult, IncomingLikeDecision, LANGUAGE_PROMPT, Language, SenderInfo, State,
    profile_language,
};
use crate::db::profile_repository::ProfileRow;
use crate::domain::{Gender, Profile};
use crate::telegram::i18n::TextKey;
use crate::telegram::keyboards::{
    make_gender_keyboard, make_incoming_like_keyboard, make_language_keyboard,
    make_main_menu_keyboard, make_profile_action_keyboard, make_profile_confirmation_keyboard,
    make_settings_keyboard,
};
use teloxide::prelude::*;
use teloxide::types::{FileId, InputFile, KeyboardMarkup, ReplyMarkup};

#[allow(clippy::needless_pass_by_value)]
pub fn is_incoming_like_decision_message(msg: Message) -> bool {
    msg.text().and_then(IncomingLikeDecision::parse).is_some()
}

#[allow(clippy::needless_pass_by_value)]
pub fn is_start_command(msg: Message) -> bool {
    msg.text().is_some_and(|text| is_command(text, "start"))
}

#[allow(clippy::needless_pass_by_value)]
pub fn is_language_command(msg: Message) -> bool {
    msg.text().is_some_and(|text| is_command(text, "language"))
}

#[allow(clippy::needless_pass_by_value)]
pub fn is_my_profile_command(msg: Message) -> bool {
    msg.text().is_some_and(|text| is_command(text, "myprofile"))
}

pub async fn prompt_for_language_selection(
    bot: &Bot,
    chat_id: ChatId,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, LANGUAGE_PROMPT)
        .reply_markup(make_language_keyboard())
        .await?;

    Ok(())
}

pub async fn require_sender(
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
        telegram_user_id: i64::try_from(user.id.0).expect("Telegram user id does not fit into i64"),
        telegram_username: user.username.clone(),
    }))
}

pub async fn send_profile_card(
    bot: &Bot,
    chat_id: ChatId,
    profile: &Profile,
    header_text: Option<&str>,
    reply_markup: Option<ReplyMarkup>,
) -> HandlerResult {
    let mut text = String::new();

    if let Some(header) = header_text {
        text.push_str(header);
        text.push_str("\n\n");
    }

    text.push_str(&profile.display_text());

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

pub async fn show_settings_menu(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, language.text(TextKey::SettingsMenuText))
        .reply_markup(make_settings_keyboard(language))
        .await?;

    Ok(())
}

pub async fn show_main_menu(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, language.text(TextKey::MainMenuText))
        .reply_markup(make_main_menu_keyboard(language))
        .await?;

    Ok(())
}

pub async fn prompt_for_swipe_decision(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, language.text(TextKey::LikeOrSkip))
        .reply_markup(make_profile_action_keyboard())
        .await?;

    Ok(())
}

pub async fn prompt_for_incoming_like_decision(
    bot: &Bot,
    chat_id: ChatId,
    profile: &Profile,
    pending_like_count: i64,
) -> Result<(), teloxide::RequestError> {
    let language = profile_language(profile);

    bot.send_message(
        chat_id,
        build_incoming_like_prompt(profile, pending_like_count),
    )
    .reply_markup(make_incoming_like_keyboard(language))
    .await?;

    Ok(())
}

pub async fn show_profile_confirmation_preview(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
    draft_profile: &Profile,
) -> HandlerResult {
    send_profile_card(bot, chat_id, draft_profile, None, None).await?;
    bot.send_message(chat_id, language.text(TextKey::SaveThisProfile))
        .reply_markup(make_profile_confirmation_keyboard(language))
        .await?;

    Ok(())
}

pub async fn show_candidate_profile(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    current_user_profile: Profile,
    displayed_profile: &Profile,
    header_text: Option<&str>,
    return_to_main_menu: bool,
) -> HandlerResult {
    let displayed_profile_user_id = displayed_profile.telegram_user_id.unwrap_or(0);

    send_profile_card(
        bot,
        chat_id,
        displayed_profile,
        header_text,
        Some(make_profile_action_keyboard().into()),
    )
    .await?;

    dialogue
        .update(State::AwaitingProfileAction {
            profile: current_user_profile,
            displayed_profile_user_id,
            return_to_main_menu,
        })
        .await?;

    Ok(())
}

pub async fn transition_to_main_menu(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    profile: Profile,
) -> HandlerResult {
    show_main_menu(bot, chat_id, profile_language(&profile)).await?;
    dialogue.update(State::MainMenu { profile }).await?;

    Ok(())
}

pub async fn transition_to_edit_menu(dialogue: &AppDialogue, profile: Profile) -> HandlerResult {
    dialogue.update(State::EditMenu { profile }).await?;

    Ok(())
}

pub async fn transition_to_settings_menu(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    profile: Profile,
) -> HandlerResult {
    let language = profile_language(&profile);

    show_settings_menu(bot, chat_id, language).await?;
    dialogue.update(State::SettingsMenu { profile }).await?;

    Ok(())
}

pub fn build_gender_selection_keyboard(language: Language) -> KeyboardMarkup {
    make_gender_keyboard(language)
}

pub fn extract_photo_file_id(msg: &Message) -> Option<String> {
    msg.photo()
        .and_then(|photo_sizes| photo_sizes.last())
        .map(|largest_photo| largest_photo.file.id.to_string())
}

pub fn build_match_notification_text(profile_row: &ProfileRow, language: Language) -> String {
    profile_contact_link(profile_row).map_or_else(
        || {
            format!(
                "{} {}",
                language.text(TextKey::MatchNoUsername),
                html_escape(&profile_row.name)
            )
        },
        |link| format!("{}{}", language.text(TextKey::MatchStartChatting), link),
    )
}

fn build_incoming_like_prompt(profile: &Profile, pending_like_count: i64) -> String {
    let language = profile_language(profile);
    let is_plural = pending_like_count != 1;
    let key = match (profile.looking_for, is_plural) {
        (Some(Gender::Female), false) => TextKey::IncomingLikePromptFemaleSingular,
        (Some(Gender::Female), true) => TextKey::IncomingLikePromptFemalePlural,
        (Some(Gender::Male), false) => TextKey::IncomingLikePromptMaleSingular,
        (Some(Gender::Male), true) => TextKey::IncomingLikePromptMalePlural,
        (None, false) => TextKey::IncomingLikePromptNeutralSingular,
        (None, true) => TextKey::IncomingLikePromptNeutralPlural,
    };

    language
        .text(key)
        .replace("{count}", &pending_like_count.to_string())
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn profile_contact_link(profile_row: &ProfileRow) -> Option<String> {
    profile_row
        .username
        .as_deref()
        .filter(|username| !username.is_empty())
        .map(|username| telegram_username_link_html(username, &profile_row.name))
}

fn telegram_username_link_html(username: &str, name: &str) -> String {
    format!(
        r#"<a href="https://t.me/{}">{}</a>"#,
        html_escape(username),
        html_escape(name)
    )
}

fn is_command(text: &str, command: &str) -> bool {
    text.split_whitespace()
        .next()
        .and_then(|command_text| command_text.strip_prefix('/'))
        .and_then(|command_text| command_text.split('@').next())
        == Some(command)
}

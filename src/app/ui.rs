use crate::app::types::{
    HandlerResult, IncomingLikeDecision, LANGUAGE_PROMPT, Lang, MyDialogue, SenderInfo, State,
    profile_lang,
};
use crate::bot::i18n::TextKey;
use crate::bot::keyboards::{
    make_edit_profile_keyboard, make_gender_keyboard, make_incoming_like_keyboard,
    make_language_keyboard, make_main_menu_keyboard, make_profile_action_keyboard,
    make_profile_confirmation_keyboard, make_settings_keyboard,
};
use crate::db::profile_repository::ProfileRow;
use crate::models::Profile;
use teloxide::prelude::*;
use teloxide::types::{FileId, InputFile, KeyboardMarkup, ReplyMarkup};

pub fn is_incoming_like_decision_message(msg: Message) -> bool {
    msg.text().and_then(IncomingLikeDecision::parse).is_some()
}

pub fn is_start_command(msg: Message) -> bool {
    msg.text()
        .map(|text| text.trim_start().starts_with("/start"))
        .unwrap_or(false)
}

pub async fn prompt_language_selection(
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
        telegram_user_id: user.id.0 as i64,
        telegram_username: user.username.clone(),
    }))
}

pub async fn send_profile(
    bot: &Bot,
    chat_id: ChatId,
    profile: &Profile,
    header_text: Option<&str>,
    footer_text: Option<&str>,
    reply_markup: Option<ReplyMarkup>,
) -> HandlerResult {
    let mut text = String::new();

    if let Some(header) = header_text {
        text.push_str(header);
        text.push_str("\n\n");
    }

    text.push_str(&profile.display_text());

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

pub async fn show_edit_menu(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, lang.text(TextKey::EditMenuText))
        .reply_markup(make_edit_profile_keyboard(lang))
        .await?;

    Ok(())
}

pub async fn show_settings_menu(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, lang.text(TextKey::SettingsMenuText))
        .reply_markup(make_settings_keyboard(lang))
        .await?;

    Ok(())
}

pub async fn show_main_menu(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, lang.text(TextKey::MainMenuText))
        .reply_markup(make_main_menu_keyboard(lang))
        .await?;

    Ok(())
}

pub async fn prompt_for_profile_action(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, lang.text(TextKey::LikeOrSkip))
        .reply_markup(make_profile_action_keyboard())
        .await?;

    Ok(())
}

pub async fn prompt_incoming_like_decision(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, lang.text(TextKey::SomeoneLikedYouShowQuestion))
        .reply_markup(make_incoming_like_keyboard(lang))
        .await?;

    Ok(())
}

pub async fn preview_profile_for_confirmation(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
    draft: &Profile,
) -> HandlerResult {
    send_profile(bot, chat_id, draft, None, None, None).await?;
    bot.send_message(chat_id, lang.text(TextKey::SaveThisProfile))
        .reply_markup(make_profile_confirmation_keyboard(lang))
        .await?;

    Ok(())
}

pub async fn show_profile_for_action(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    my_profile: Profile,
    shown_profile_user_id: i64,
    shown_profile: &Profile,
    header_text: Option<&str>,
    footer_text: Option<&str>,
    return_to_menu: bool,
) -> HandlerResult {
    send_profile(
        bot,
        chat_id,
        shown_profile,
        header_text,
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

pub async fn move_to_main_menu(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    profile: Profile,
) -> HandlerResult {
    show_main_menu(bot, chat_id, profile_lang(&profile)).await?;
    dialogue.update(State::MainMenu { profile }).await?;

    Ok(())
}

pub async fn move_to_edit_menu(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    profile: Profile,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    show_edit_menu(bot, chat_id, lang).await?;
    dialogue.update(State::EditMenu { profile }).await?;

    Ok(())
}

pub async fn move_to_settings_menu(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    profile: Profile,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    show_settings_menu(bot, chat_id, lang).await?;
    dialogue.update(State::SettingsMenu { profile }).await?;

    Ok(())
}

pub fn make_gender_prompt_keyboard(lang: Lang) -> KeyboardMarkup {
    make_gender_keyboard(lang)
}

pub fn extract_photo_file_id(msg: &Message) -> Option<String> {
    msg.photo()
        .and_then(|photo_sizes| photo_sizes.last())
        .map(|largest_photo| largest_photo.file.id.to_string())
}

pub fn match_notification_text(profile_row: &ProfileRow, lang: Lang) -> String {
    match profile_contact_link(profile_row) {
        Some(link) => format!("{}{}", lang.text(TextKey::MatchStartChatting), link),
        None => format!(
            "{} {}",
            lang.text(TextKey::MatchNoUsername),
            html_escape(&profile_row.name)
        ),
    }
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

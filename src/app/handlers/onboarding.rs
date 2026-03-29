use crate::app::types::{
    ConfirmationAction, EditMenuAction, HandlerResult, Lang, MyDialogue, State, profile_lang,
};
use crate::app::ui::{
    extract_photo_file_id, make_gender_prompt_keyboard, move_to_edit_menu, move_to_main_menu,
    preview_profile_for_confirmation, prompt_incoming_like_decision, prompt_language_selection,
    require_sender, send_profile, show_edit_menu,
};
use crate::bot::i18n::TextKey;
use crate::bot::keyboards::make_profile_confirmation_keyboard;
use crate::db::profile_repository::{get_profile_by_user_id, save_profile};
use crate::db::swipe_repository::get_incoming_like_for_user;
use crate::models::{CompleteProfile, Gender, Profile};
use sqlx::PgPool;
use teloxide::prelude::*;

pub async fn global_start(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
) -> HandlerResult {
    let Some(sender) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    if let Some(profile_row) = get_profile_by_user_id(&pool, sender.telegram_user_id).await? {
        open_existing_profile_home(
            &bot,
            &dialogue,
            &msg,
            sender.telegram_user_id,
            profile_row.to_profile(),
            &pool,
        )
        .await?;
    } else {
        enter_language_selection(&bot, &dialogue, msg.chat.id).await?;
    }

    Ok(())
}

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    enter_language_selection(&bot, &dialogue, msg.chat.id).await
}

pub async fn receive_language(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        prompt_language_selection(&bot, msg.chat.id).await?;
        return Ok(());
    };

    let Some(lang) = Lang::from_text(text) else {
        prompt_language_selection(&bot, msg.chat.id).await?;
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

pub async fn receive_name(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);
    let Some(name) = msg.text().map(str::trim).filter(|text| !text.is_empty()) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::NonEmptyName))
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

    bot.send_message(msg.chat.id, lang.text(TextKey::SelectGender))
        .reply_markup(make_gender_prompt_keyboard(lang))
        .await?;
    dialogue.update(State::WaitingForGender { draft }).await?;

    Ok(())
}

pub async fn receive_gender(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, lang.text(TextKey::SelectGender))
            .reply_markup(make_gender_prompt_keyboard(lang))
            .await?;
        return Ok(());
    };

    let Some(gender) = Gender::from_text(text, lang) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::SelectGender))
            .reply_markup(make_gender_prompt_keyboard(lang))
            .await?;
        return Ok(());
    };

    draft.gender = Some(gender);

    bot.send_message(msg.chat.id, lang.text(TextKey::LookingFor))
        .reply_markup(make_gender_prompt_keyboard(lang))
        .await?;
    dialogue
        .update(State::WaitingForLookingFor { draft })
        .await?;

    Ok(())
}

pub async fn receive_looking_for(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, lang.text(TextKey::LookingFor))
            .reply_markup(make_gender_prompt_keyboard(lang))
            .await?;
        return Ok(());
    };

    let Some(looking_for) = Gender::from_text(text, lang) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::LookingFor))
            .reply_markup(make_gender_prompt_keyboard(lang))
            .await?;
        return Ok(());
    };

    draft.looking_for = Some(looking_for);

    bot.send_message(msg.chat.id, lang.text(TextKey::AskAge))
        .await?;
    dialogue.update(State::WaitingForAge { draft }).await?;

    Ok(())
}

pub async fn receive_age(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(age) = msg.text().and_then(parse_age) else {
        bot.send_message(msg.chat.id, lang.text(TextKey::AskAge))
            .await?;
        return Ok(());
    };

    draft.age = Some(age);

    bot.send_message(msg.chat.id, lang.text(TextKey::AskLocation))
        .await?;
    dialogue.update(State::WaitingForLocation { draft }).await?;

    Ok(())
}

pub async fn receive_location(
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

pub async fn receive_description(
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

pub async fn receive_photo(
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

    preview_profile_for_confirmation(&bot, msg.chat.id, lang, &draft).await?;
    dialogue.update(State::ConfirmProfile { draft }).await?;

    Ok(())
}

pub async fn confirm_profile(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    draft: Profile,
    pool: PgPool,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, lang.text(TextKey::UseKeyboardToConfirm))
            .reply_markup(make_profile_confirmation_keyboard(lang))
            .await?;
        return Ok(());
    };

    match ConfirmationAction::parse(text) {
        Some(ConfirmationAction::SaveProfile) => {
            save_current_draft(&bot, &dialogue, &msg, draft, &pool).await?;
        }
        Some(ConfirmationAction::EditProfile) => {
            move_to_edit_menu(&bot, &dialogue, msg.chat.id, draft).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Choose one of the available actions.")
                .reply_markup(make_profile_confirmation_keyboard(lang))
                .await?;
        }
    }

    Ok(())
}

pub async fn edit_profile_menu(
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
            bot.send_message(msg.chat.id, lang.text(TextKey::RebuildProfile))
                .await?;
            dialogue
                .update(State::WaitingForName { draft: profile })
                .await?;
        }
        Some(EditMenuAction::ChangePhoto) => {
            bot.send_message(msg.chat.id, lang.text(TextKey::AskPhoto))
                .await?;
            dialogue
                .update(State::WaitingForPhotoEdit { draft: profile })
                .await?;
        }
        Some(EditMenuAction::ChangeBio) => {
            bot.send_message(msg.chat.id, lang.text(TextKey::AskBio))
                .await?;
            dialogue
                .update(State::WaitingForDescriptionEdit { draft: profile })
                .await?;
        }
        Some(EditMenuAction::BackToMainMenu) => {
            move_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
        }
        None => {
            show_edit_menu(&bot, msg.chat.id, lang).await?;
        }
    }

    Ok(())
}

pub async fn edit_photo(
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

pub async fn edit_description(
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

    preview_profile_for_confirmation(&bot, msg.chat.id, lang, &draft).await?;
    dialogue.update(State::ConfirmProfile { draft }).await?;

    Ok(())
}

async fn enter_language_selection(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
) -> HandlerResult {
    prompt_language_selection(bot, chat_id).await?;
    dialogue.update(State::WaitingForLanguage).await?;

    Ok(())
}

async fn open_existing_profile_home(
    bot: &Bot,
    dialogue: &MyDialogue,
    msg: &Message,
    user_id: i64,
    profile: Profile,
    pool: &PgPool,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    send_profile(bot, msg.chat.id, &profile, None, None, None).await?;

    if let Some(incoming_like_row) = get_incoming_like_for_user(pool, user_id).await? {
        prompt_incoming_like_decision(bot, msg.chat.id, lang).await?;
        dialogue
            .update(State::AwaitingIncomingLikeDecision {
                profile,
                liked_you_user_id: incoming_like_row.telegram_user_id,
            })
            .await?;
    } else {
        move_to_main_menu(bot, dialogue, msg.chat.id, profile).await?;
    }

    Ok(())
}

async fn save_current_draft(
    bot: &Bot,
    dialogue: &MyDialogue,
    msg: &Message,
    mut draft: Profile,
    pool: &PgPool,
) -> HandlerResult {
    let Some(sender) = require_sender(bot, msg).await? else {
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

    save_profile(pool, &complete_profile).await?;

    log::info!("Saved profile for user {}", sender.telegram_user_id);
    move_to_main_menu(bot, dialogue, msg.chat.id, draft).await?;

    Ok(())
}

fn parse_age(text: &str) -> Option<u8> {
    let age = text.trim().parse::<u8>().ok()?;
    (1..=100).contains(&age).then_some(age)
}

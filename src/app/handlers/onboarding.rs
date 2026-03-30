use super::resume_main_menu;
use crate::app::types::{
    ConfirmationAction, EditMenuAction, HandlerResult, Lang, MyDialogue, State, profile_lang,
};
use crate::app::ui::{
    extract_photo_file_id, make_gender_prompt_keyboard, move_to_edit_menu, move_to_main_menu,
    preview_profile_for_confirmation, prompt_incoming_like_decision, prompt_language_selection,
    require_sender, send_profile, show_edit_menu,
};
use crate::bot::i18n::TextKey;
use crate::bot::keyboards::{
    make_keep_previous_photo_keyboard, make_location_choice_keyboard, make_location_keyboard,
    make_previous_value_keyboard, make_profile_confirmation_keyboard, make_skip_keyboard,
};
use crate::db::profile_repository::{get_profile_by_user_id, save_profile};
use crate::db::swipe_repository::get_incoming_like_for_user;
use crate::models::{CompleteProfile, Gender, Profile};
use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::KeyboardMarkup;

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

pub async fn start(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
) -> HandlerResult {
    let Some(sender) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    if let Some(profile_row) = get_profile_by_user_id(&pool, sender.telegram_user_id).await? {
        resume_main_menu(bot, dialogue, msg, profile_row.to_profile(), pool).await?;
    } else {
        enter_language_selection(&bot, &dialogue, msg.chat.id).await?;
    }

    Ok(())
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

    prompt_for_name_input(
        &bot,
        msg.chat.id,
        lang,
        TextKey::WhatIsYourName,
        draft.name.as_deref(),
    )
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
        prompt_for_name_input(
            &bot,
            msg.chat.id,
            lang,
            TextKey::NonEmptyName,
            draft.name.as_deref(),
        )
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

    prompt_for_age_input(&bot, msg.chat.id, lang, draft.age).await?;
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
        prompt_for_age_input(&bot, msg.chat.id, lang, draft.age).await?;
        return Ok(());
    };

    draft.age = Some(age);

    prompt_for_location_input(&bot, msg.chat.id, lang, draft.location.as_deref()).await?;
    dialogue.update(State::WaitingForLocation { draft }).await?;

    Ok(())
}

async fn prompt_for_location_input(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
    previous_location: Option<&str>,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, lang.text(TextKey::AskLocation))
        .reply_markup(make_location_keyboard(lang, previous_location))
        .await?;

    Ok(())
}

pub async fn receive_location(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut draft: Profile,
    geocoder: crate::geocoding::Geocoder,
) -> HandlerResult {
    let lang = profile_lang(&draft);

    // if user sent location via Telegram's location sharing
    if let Some(shared_location) = msg.location() {
        let Some(city) = geocoder
            .reverse_geocode_city(shared_location.latitude, shared_location.longitude, lang)
            .await?
        else {
            prompt_for_location_input(&bot, msg.chat.id, lang, draft.location.as_deref()).await?;
            return Ok(());
        };

        draft.location = Some(city.label);
        draft.latitude = Some(shared_location.latitude);
        draft.longitude = Some(shared_location.longitude);

        log::info!(
            "User {} shared location, resolved city: {}, latitude: {}, longitude: {}",
            draft.telegram_user_id.unwrap_or_default(),
            draft.location.as_deref().unwrap_or_default(), draft.latitude.unwrap_or_default(), draft.longitude.unwrap_or_default()
        );

        prompt_for_description_input(
            &bot,
            msg.chat.id,
            lang,
            TextKey::AskBio,
            draft.description.as_deref(),
        )
        .await?;
        dialogue
            .update(State::WaitingForDescription { draft })
            .await?;

        return Ok(());
    }

    // else - user sent location as text, try to geocode it
    let Some(query) = msg.text().map(str::trim).filter(|text| !text.is_empty()) else {
        prompt_for_location_input(&bot, msg.chat.id, lang, draft.location.as_deref()).await?;
        return Ok(());
    };

    if should_keep_existing_location(query, &draft) {
        prompt_for_description_input(
            &bot,
            msg.chat.id,
            lang,
            TextKey::AskBio,
            draft.description.as_deref(),
        )
        .await?;
        dialogue
            .update(State::WaitingForDescription { draft })
            .await?;
        return Ok(());
    }

    let candidates = geocoder.search_cities(query, lang).await?;

    match candidates.len() {
        0 => {
            prompt_for_location_input(&bot, msg.chat.id, lang, draft.location.as_deref()).await?;
        }
        1 => {
            let candidate = &candidates[0];
            draft.location = Some(candidate.label.clone());
            draft.latitude = Some(candidate.latitude);
            draft.longitude = Some(candidate.longitude);

            prompt_for_description_input(
                &bot,
                msg.chat.id,
                lang,
                TextKey::AskBio,
                draft.description.as_deref(),
            )
            .await?;
            dialogue
                .update(State::WaitingForDescription { draft })
                .await?;
        }
        _ => {
            let text = candidates
                .iter()
                .enumerate()
                .map(|(index, candidate)| format!("{}. {}", index + 1, candidate.label))
                .collect::<Vec<_>>()
                .join("\n");

            bot.send_message(msg.chat.id, text)
                .reply_markup(make_location_choice_keyboard(candidates.len()))
                .await?;

            dialogue
                .update(State::WaitingForLocationChoice { draft, candidates })
                .await?;
        }
    }

    Ok(())
}

pub async fn receive_location_choice(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (mut draft, candidates): (Profile, Vec<crate::geocoding::CityCandidate>),
) -> HandlerResult {
    let lang = profile_lang(&draft);

    let Some(choice) = msg
        .text()
        .map(str::trim)
        .and_then(|text| text.parse::<usize>().ok())
        .filter(|choice| *choice >= 1 && *choice <= candidates.len())
    else {
        let text = candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| format!("{}. {}", index + 1, candidate.label))
            .collect::<Vec<_>>()
            .join("\n");

        bot.send_message(msg.chat.id, text)
            .reply_markup(make_location_choice_keyboard(candidates.len()))
            .await?;

        dialogue
            .update(State::WaitingForLocationChoice { draft, candidates })
            .await?;
        return Ok(());
    };

    let candidate = &candidates[choice - 1];
    draft.location = Some(candidate.label.clone());
    draft.latitude = Some(candidate.latitude);
    draft.longitude = Some(candidate.longitude);

    prompt_for_description_input(
        &bot,
        msg.chat.id,
        lang,
        TextKey::AskBio,
        draft.description.as_deref(),
    )
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
    let Some(description) = description_input(msg.text(), lang) else {
        prompt_for_description_input(
            &bot,
            msg.chat.id,
            lang,
            TextKey::NonEmptyBio,
            draft.description.as_deref(),
        )
        .await?;
        return Ok(());
    };

    draft.description = Some(description);

    prompt_for_photo_input(
        &bot,
        msg.chat.id,
        lang,
        TextKey::AskPhoto,
        draft.photo.as_deref(),
    )
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
    let Some(photo_file_id) =
        extract_photo_file_id(&msg).or_else(|| kept_photo_file_id(msg.text(), lang, &draft))
    else {
        prompt_for_photo_input(
            &bot,
            msg.chat.id,
            lang,
            TextKey::AskPhoto,
            draft.photo.as_deref(),
        )
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
            prompt_for_name_input(
                &bot,
                msg.chat.id,
                lang,
                TextKey::RebuildProfile,
                profile.name.as_deref(),
            )
            .await?;
            dialogue
                .update(State::WaitingForName { draft: profile })
                .await?;
        }
        Some(EditMenuAction::ChangePhoto) => {
            prompt_for_photo_input(
                &bot,
                msg.chat.id,
                lang,
                TextKey::AskPhoto,
                profile.photo.as_deref(),
            )
            .await?;
            dialogue
                .update(State::WaitingForPhotoEdit { draft: profile })
                .await?;
        }
        Some(EditMenuAction::ChangeBio) => {
            prompt_for_description_input(
                &bot,
                msg.chat.id,
                lang,
                TextKey::AskBio,
                profile.description.as_deref(),
            )
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
    let Some(photo_file_id) =
        extract_photo_file_id(&msg).or_else(|| kept_photo_file_id(msg.text(), lang, &draft))
    else {
        prompt_for_photo_input(
            &bot,
            msg.chat.id,
            lang,
            TextKey::SendPhotoMessage,
            draft.photo.as_deref(),
        )
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
    let Some(description) = description_input(msg.text(), lang) else {
        prompt_for_description_input(
            &bot,
            msg.chat.id,
            lang,
            TextKey::NonEmptyBio,
            draft.description.as_deref(),
        )
        .await?;
        return Ok(());
    };

    draft.description = Some(description);

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

async fn prompt_for_name_input(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
    prompt_key: TextKey,
    previous_name: Option<&str>,
) -> Result<(), teloxide::RequestError> {
    send_message_with_optional_keyboard(
        bot,
        chat_id,
        lang.text(prompt_key),
        previous_value_keyboard(previous_name),
    )
    .await
}

async fn prompt_for_age_input(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
    previous_age: Option<u8>,
) -> Result<(), teloxide::RequestError> {
    send_message_with_optional_keyboard(
        bot,
        chat_id,
        lang.text(TextKey::AskAge),
        previous_age.map(|age| make_previous_value_keyboard(age.to_string())),
    )
    .await
}

async fn prompt_for_description_input(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
    prompt_key: TextKey,
    previous_description: Option<&str>,
) -> Result<(), teloxide::RequestError> {
    send_message_with_optional_keyboard(
        bot,
        chat_id,
        lang.text(prompt_key),
        description_keyboard(lang, previous_description),
    )
    .await
}

async fn prompt_for_photo_input(
    bot: &Bot,
    chat_id: ChatId,
    lang: Lang,
    prompt_key: TextKey,
    previous_photo: Option<&str>,
) -> Result<(), teloxide::RequestError> {
    send_message_with_optional_keyboard(
        bot,
        chat_id,
        lang.text(prompt_key),
        non_empty_value(previous_photo).map(|_| make_keep_previous_photo_keyboard(lang)),
    )
    .await
}

async fn send_message_with_optional_keyboard(
    bot: &Bot,
    chat_id: ChatId,
    text: &str,
    keyboard: Option<KeyboardMarkup>,
) -> Result<(), teloxide::RequestError> {
    match keyboard {
        Some(keyboard) => {
            bot.send_message(chat_id, text)
                .reply_markup(keyboard)
                .await?;
        }
        None => {
            bot.send_message(chat_id, text).await?;
        }
    }

    Ok(())
}

fn previous_value_keyboard(value: Option<&str>) -> Option<KeyboardMarkup> {
    non_empty_value(value).map(make_previous_value_keyboard)
}

fn description_keyboard(lang: Lang, previous_description: Option<&str>) -> Option<KeyboardMarkup> {
    previous_value_keyboard(previous_description).or_else(|| Some(make_skip_keyboard(lang)))
}

fn non_empty_value(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn description_input(text: Option<&str>, lang: Lang) -> Option<String> {
    match text.map(str::trim) {
        Some(text) if text == lang.text(TextKey::SkipInput) => Some(String::new()),
        Some(text) if !text.is_empty() => Some(text.to_owned()),
        _ => None,
    }
}

fn should_keep_existing_location(query: &str, draft: &Profile) -> bool {
    matches!(
        non_empty_value(draft.location.as_deref()),
        Some(location)
            if location == query.trim() && draft.latitude.is_some() && draft.longitude.is_some()
    )
}

fn kept_photo_file_id(text: Option<&str>, lang: Lang, draft: &Profile) -> Option<String> {
    if text.map(str::trim) == Some(lang.text(TextKey::KeepPreviousPhoto)) {
        non_empty_value(draft.photo.as_deref()).map(ToOwned::to_owned)
    } else {
        None
    }
}

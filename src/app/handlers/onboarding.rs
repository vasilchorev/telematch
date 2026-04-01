use super::resume_main_menu_flow;
use crate::app::chat_ui::{
    build_gender_selection_keyboard, extract_photo_file_id, prompt_for_incoming_like_decision,
    prompt_for_language_selection, require_sender, send_profile_card,
    show_profile_confirmation_preview, transition_to_main_menu,
};
use crate::app::types::{
    AppDialogue, ConfirmationAction, EditMenuAction, HandlerResult, Language, SenderInfo, State,
    profile_language,
};
use crate::db::profile_repository::{get_profile_by_user_id, save_profile};
use crate::db::swipe_repository::get_pending_incoming_like_target_for_user;
use crate::domain::{CompleteProfile, Gender, Profile};
use crate::services::{CityCandidate, Geocoder};
use crate::telegram::i18n::TextKey;
use crate::telegram::keyboards::{
    make_keep_previous_photo_keyboard, make_location_choice_keyboard, make_location_keyboard,
    make_previous_value_keyboard, make_profile_confirmation_keyboard, make_skip_keyboard,
};
use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::types::KeyboardMarkup;

pub async fn handle_start_command(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    pool: PgPool,
) -> HandlerResult {
    let Some(sender_info) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    if let Some(profile_row) = get_profile_by_user_id(&pool, sender_info.telegram_user_id).await? {
        open_existing_profile_home(
            &bot,
            &dialogue,
            &msg,
            sender_info.telegram_user_id,
            profile_row.to_profile(),
            &pool,
        )
        .await?;
    } else {
        enter_language_selection(&bot, &dialogue, msg.chat.id).await?;
    }

    Ok(())
}

pub async fn handle_initial_state(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    pool: PgPool,
) -> HandlerResult {
    let Some(sender_info) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    if let Some(profile_row) = get_profile_by_user_id(&pool, sender_info.telegram_user_id).await? {
        resume_main_menu_flow(bot, dialogue, msg, profile_row.to_profile(), pool).await?;
    } else {
        enter_language_selection(&bot, &dialogue, msg.chat.id).await?;
    }

    Ok(())
}

pub async fn handle_language_selection(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        prompt_for_language_selection(&bot, msg.chat.id).await?;
        return Ok(());
    };

    let Some(language) = Language::from_text(text) else {
        prompt_for_language_selection(&bot, msg.chat.id).await?;
        return Ok(());
    };

    let draft = Profile {
        chat_id: Some(msg.chat.id.0),
        language_code: Some(language.as_db_code().to_owned()),
        ..Profile::default()
    };

    prompt_for_name_input(
        &bot,
        msg.chat.id,
        language,
        TextKey::WhatIsYourName,
        draft.name.as_deref(),
    )
    .await?;
    dialogue.update(State::WaitingForName { draft }).await?;

    Ok(())
}

pub async fn handle_name_input(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let language = profile_language(&draft);
    let Some(name) = msg.text().map(str::trim).filter(|text| !text.is_empty()) else {
        prompt_for_name_input(
            &bot,
            msg.chat.id,
            language,
            TextKey::NonEmptyName,
            draft.name.as_deref(),
        )
        .await?;
        return Ok(());
    };

    let Some(sender_info) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    apply_sender_info(&mut draft, msg.chat.id, sender_info);
    draft.name = Some(name.to_owned());

    prompt_for_gender_selection(&bot, msg.chat.id, language, TextKey::SelectGender).await?;
    dialogue.update(State::WaitingForGender { draft }).await?;

    Ok(())
}

pub async fn handle_gender_selection(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let language = profile_language(&draft);

    let Some(text) = msg.text() else {
        prompt_for_gender_selection(&bot, msg.chat.id, language, TextKey::SelectGender).await?;
        return Ok(());
    };

    let Some(gender) = Gender::from_text(text, language) else {
        prompt_for_gender_selection(&bot, msg.chat.id, language, TextKey::SelectGender).await?;
        return Ok(());
    };

    draft.gender = Some(gender);

    prompt_for_gender_selection(&bot, msg.chat.id, language, TextKey::LookingFor).await?;
    dialogue
        .update(State::WaitingForLookingFor { draft })
        .await?;

    Ok(())
}

pub async fn handle_match_preference_selection(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let language = profile_language(&draft);

    let Some(text) = msg.text() else {
        prompt_for_gender_selection(&bot, msg.chat.id, language, TextKey::LookingFor).await?;
        return Ok(());
    };

    let Some(looking_for) = Gender::from_text(text, language) else {
        prompt_for_gender_selection(&bot, msg.chat.id, language, TextKey::LookingFor).await?;
        return Ok(());
    };

    draft.looking_for = Some(looking_for);

    prompt_for_age_input(&bot, msg.chat.id, language, draft.age).await?;
    dialogue.update(State::WaitingForAge { draft }).await?;

    Ok(())
}

pub async fn handle_age_input(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    mut draft: Profile,
) -> HandlerResult {
    let language = profile_language(&draft);

    let Some(age) = msg.text().and_then(parse_age) else {
        prompt_for_age_input(&bot, msg.chat.id, language, draft.age).await?;
        return Ok(());
    };

    draft.age = Some(age);

    prompt_for_location_input(&bot, msg.chat.id, language, draft.location.as_deref()).await?;
    dialogue.update(State::WaitingForLocation { draft }).await?;

    Ok(())
}

async fn prompt_for_location_input(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
    previous_location: Option<&str>,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, language.text(TextKey::AskLocation))
        .reply_markup(make_location_keyboard(language, previous_location))
        .await?;

    Ok(())
}

pub async fn handle_location_input(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    mut draft: Profile,
    geocoder: Geocoder,
) -> HandlerResult {
    let language = profile_language(&draft);

    if let Some(shared_location) = msg.location() {
        let Some(city) = geocoder
            .reverse_geocode_city(
                shared_location.latitude,
                shared_location.longitude,
                language,
            )
            .await?
        else {
            prompt_for_location_input(&bot, msg.chat.id, language, draft.location.as_deref())
                .await?;
            return Ok(());
        };

        set_location(
            &mut draft,
            city.label,
            shared_location.latitude,
            shared_location.longitude,
        );
        log_location_resolution(&draft);

        return transition_to_description_step(&bot, &dialogue, msg.chat.id, draft).await;
    }

    let Some(query) = msg.text().map(str::trim).filter(|text| !text.is_empty()) else {
        prompt_for_location_input(&bot, msg.chat.id, language, draft.location.as_deref()).await?;
        return Ok(());
    };

    if should_keep_existing_location(query, &draft) {
        return transition_to_description_step(&bot, &dialogue, msg.chat.id, draft).await;
    }

    let candidates = geocoder.search_cities(query, language).await?;

    match candidates.as_slice() {
        [] => {
            prompt_for_location_input(&bot, msg.chat.id, language, draft.location.as_deref())
                .await?;
        }
        [candidate] => {
            apply_city_candidate(&mut draft, candidate);
            transition_to_description_step(&bot, &dialogue, msg.chat.id, draft).await?;
        }
        _ => {
            prompt_for_location_choice(&bot, msg.chat.id, &candidates).await?;
            dialogue
                .update(State::WaitingForLocationChoice { draft, candidates })
                .await?;
        }
    }

    Ok(())
}

pub async fn handle_location_choice_selection(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    (mut draft, candidates): (Profile, Vec<CityCandidate>),
) -> HandlerResult {
    let Some(choice) = parse_location_choice(msg.text(), candidates.len()) else {
        prompt_for_location_choice(&bot, msg.chat.id, &candidates).await?;
        dialogue
            .update(State::WaitingForLocationChoice { draft, candidates })
            .await?;
        return Ok(());
    };

    apply_city_candidate(&mut draft, &candidates[choice - 1]);
    transition_to_description_step(&bot, &dialogue, msg.chat.id, draft).await
}

pub async fn handle_description_step(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    draft: Profile,
) -> HandlerResult {
    process_description_input(&bot, &dialogue, &msg, draft, DescriptionNextStep::Photo).await
}

pub async fn handle_photo_step(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    draft: Profile,
) -> HandlerResult {
    process_photo_input(&bot, &dialogue, &msg, draft, TextKey::AskPhoto).await
}

pub async fn handle_profile_confirmation(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    draft: Profile,
    pool: PgPool,
) -> HandlerResult {
    let language = profile_language(&draft);

    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, language.text(TextKey::UseKeyboardToConfirm))
            .reply_markup(make_profile_confirmation_keyboard(language))
            .await?;
        return Ok(());
    };

    match ConfirmationAction::parse(text) {
        Some(ConfirmationAction::SaveProfile) => {
            save_current_draft(&bot, &dialogue, &msg, draft, &pool).await?;
        }
        Some(ConfirmationAction::EditProfile) => {
            prompt_for_name_input(
                &bot,
                msg.chat.id,
                language,
                TextKey::RebuildProfile,
                draft.name.as_deref(),
            )
            .await?;
            dialogue.update(State::WaitingForName { draft }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Choose one of the available actions.")
                .reply_markup(make_profile_confirmation_keyboard(language))
                .await?;
        }
    }

    Ok(())
}

pub async fn handle_edit_menu(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    profile: Profile,
) -> HandlerResult {
    let language = profile_language(&profile);

    let Some(text) = msg.text() else {
        return Ok(());
    };

    match EditMenuAction::parse(text) {
        Some(EditMenuAction::EditProfile) => {
            prompt_for_name_input(
                &bot,
                msg.chat.id,
                language,
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
                language,
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
                language,
                TextKey::AskBio,
                profile.description.as_deref(),
            )
            .await?;
            dialogue
                .update(State::WaitingForDescriptionEdit { draft: profile })
                .await?;
        }
        Some(EditMenuAction::BackToMainMenu) => {
            transition_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
        }
        None => {}
    }

    Ok(())
}

pub async fn handle_photo_edit(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    draft: Profile,
) -> HandlerResult {
    process_photo_input(&bot, &dialogue, &msg, draft, TextKey::SendPhotoMessage).await
}

pub async fn handle_description_edit(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    draft: Profile,
) -> HandlerResult {
    process_description_input(
        &bot,
        &dialogue,
        &msg,
        draft,
        DescriptionNextStep::Confirmation,
    )
    .await
}

async fn enter_language_selection(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
) -> HandlerResult {
    prompt_for_language_selection(bot, chat_id).await?;
    dialogue.update(State::WaitingForLanguage).await?;

    Ok(())
}

async fn open_existing_profile_home(
    bot: &Bot,
    dialogue: &AppDialogue,
    msg: &Message,
    user_id: i64,
    profile: Profile,
    pool: &PgPool,
) -> HandlerResult {
    send_profile_card(bot, msg.chat.id, &profile, None, None).await?;

    if let Some(pending_target) = get_pending_incoming_like_target_for_user(pool, user_id).await? {
        prompt_for_incoming_like_decision(
            bot,
            msg.chat.id,
            &profile,
            pending_target.pending_like_count,
        )
        .await?;
        dialogue
            .update(State::AwaitingIncomingLikeDecision {
                profile,
                incoming_like_user_id: pending_target.profile_row.telegram_user_id,
                pending_like_count: pending_target.pending_like_count,
                target_kind: pending_target.target_kind,
            })
            .await?;
    } else {
        transition_to_main_menu(bot, dialogue, msg.chat.id, profile).await?;
    }

    Ok(())
}

async fn save_current_draft(
    bot: &Bot,
    dialogue: &AppDialogue,
    msg: &Message,
    mut draft: Profile,
    pool: &PgPool,
) -> HandlerResult {
    let Some(sender_info) = require_sender(bot, msg).await? else {
        return Ok(());
    };

    let sender_user_id = sender_info.telegram_user_id;
    apply_sender_info(&mut draft, msg.chat.id, sender_info);

    let complete_profile = match CompleteProfile::try_from(&draft) {
        Ok(profile) => profile,
        Err(error) => {
            log::warn!(
                "Refusing to save incomplete profile for user {}: {}",
                sender_user_id,
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

    log::info!("Saved profile for user {}", sender_user_id);
    transition_to_main_menu(bot, dialogue, msg.chat.id, draft).await?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum DescriptionNextStep {
    Photo,
    Confirmation,
}

fn parse_age(text: &str) -> Option<u8> {
    let age = text.trim().parse::<u8>().ok()?;
    (1..=100).contains(&age).then_some(age)
}

async fn prompt_for_gender_selection(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
    prompt_key: TextKey,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, language.text(prompt_key))
        .reply_markup(build_gender_selection_keyboard(language))
        .await?;

    Ok(())
}

async fn prompt_for_name_input(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
    prompt_key: TextKey,
    previous_name: Option<&str>,
) -> Result<(), teloxide::RequestError> {
    send_message_with_optional_keyboard(
        bot,
        chat_id,
        language.text(prompt_key),
        previous_value_keyboard(previous_name),
    )
    .await
}

async fn prompt_for_age_input(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
    previous_age: Option<u8>,
) -> Result<(), teloxide::RequestError> {
    send_message_with_optional_keyboard(
        bot,
        chat_id,
        language.text(TextKey::AskAge),
        previous_age.map(|age| make_previous_value_keyboard(age.to_string())),
    )
    .await
}

async fn prompt_for_location_choice(
    bot: &Bot,
    chat_id: ChatId,
    candidates: &[CityCandidate],
) -> Result<(), teloxide::RequestError> {
    bot.send_message(chat_id, format_location_candidates(candidates))
        .reply_markup(make_location_choice_keyboard(candidates.len()))
        .await?;

    Ok(())
}

async fn prompt_for_description_input(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
    prompt_key: TextKey,
    previous_description: Option<&str>,
) -> Result<(), teloxide::RequestError> {
    send_message_with_optional_keyboard(
        bot,
        chat_id,
        language.text(prompt_key),
        description_keyboard(language, previous_description),
    )
    .await
}

async fn prompt_for_photo_input(
    bot: &Bot,
    chat_id: ChatId,
    language: Language,
    prompt_key: TextKey,
    previous_photo: Option<&str>,
) -> Result<(), teloxide::RequestError> {
    send_message_with_optional_keyboard(
        bot,
        chat_id,
        language.text(prompt_key),
        non_empty_value(previous_photo).map(|_| make_keep_previous_photo_keyboard(language)),
    )
    .await
}

async fn transition_to_description_step(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    draft: Profile,
) -> HandlerResult {
    let language = profile_language(&draft);

    prompt_for_description_input(
        bot,
        chat_id,
        language,
        TextKey::AskBio,
        draft.description.as_deref(),
    )
    .await?;
    dialogue
        .update(State::WaitingForDescription { draft })
        .await?;

    Ok(())
}

async fn transition_to_confirmation_step(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    draft: Profile,
) -> HandlerResult {
    let language = profile_language(&draft);

    show_profile_confirmation_preview(bot, chat_id, language, &draft).await?;
    dialogue.update(State::ConfirmProfile { draft }).await?;

    Ok(())
}

async fn process_description_input(
    bot: &Bot,
    dialogue: &AppDialogue,
    msg: &Message,
    mut draft: Profile,
    next_step: DescriptionNextStep,
) -> HandlerResult {
    let language = profile_language(&draft);
    let Some(description) = description_input(msg.text(), language) else {
        prompt_for_description_input(
            bot,
            msg.chat.id,
            language,
            TextKey::NonEmptyBio,
            draft.description.as_deref(),
        )
        .await?;
        return Ok(());
    };

    draft.description = Some(description);

    match next_step {
        DescriptionNextStep::Photo => {
            prompt_for_photo_input(
                bot,
                msg.chat.id,
                language,
                TextKey::AskPhoto,
                draft.photo.as_deref(),
            )
            .await?;
            dialogue.update(State::WaitingForPhoto { draft }).await?;
        }
        DescriptionNextStep::Confirmation => {
            transition_to_confirmation_step(bot, dialogue, msg.chat.id, draft).await?;
        }
    }

    Ok(())
}

async fn process_photo_input(
    bot: &Bot,
    dialogue: &AppDialogue,
    msg: &Message,
    mut draft: Profile,
    retry_prompt_key: TextKey,
) -> HandlerResult {
    let language = profile_language(&draft);
    let Some(photo_file_id) =
        extract_photo_file_id(msg).or_else(|| kept_photo_file_id(msg.text(), language, &draft))
    else {
        prompt_for_photo_input(
            bot,
            msg.chat.id,
            language,
            retry_prompt_key,
            draft.photo.as_deref(),
        )
        .await?;
        return Ok(());
    };

    draft.photo = Some(photo_file_id);
    transition_to_confirmation_step(bot, dialogue, msg.chat.id, draft).await
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

fn description_keyboard(
    language: Language,
    previous_description: Option<&str>,
) -> Option<KeyboardMarkup> {
    previous_value_keyboard(previous_description).or_else(|| Some(make_skip_keyboard(language)))
}

fn non_empty_value(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn description_input(text: Option<&str>, language: Language) -> Option<String> {
    match text.map(str::trim) {
        Some(text) if text == language.text(TextKey::SkipInput) => Some(String::new()),
        Some(text) if !text.is_empty() => Some(text.to_owned()),
        _ => None,
    }
}

fn apply_sender_info(draft: &mut Profile, chat_id: ChatId, sender: SenderInfo) {
    draft.telegram_user_id = Some(sender.telegram_user_id);
    draft.chat_id = Some(chat_id.0);
    draft.username = sender.telegram_username;
}

fn apply_city_candidate(draft: &mut Profile, candidate: &CityCandidate) {
    set_location(
        draft,
        candidate.label.clone(),
        candidate.latitude,
        candidate.longitude,
    );
}

fn set_location(draft: &mut Profile, label: String, latitude: f64, longitude: f64) {
    draft.location = Some(label);
    draft.latitude = Some(latitude);
    draft.longitude = Some(longitude);
}

fn log_location_resolution(draft: &Profile) {
    log::info!(
        "User {} shared location, resolved city: {}, latitude: {}, longitude: {}",
        draft.telegram_user_id.unwrap_or_default(),
        draft.location.as_deref().unwrap_or_default(),
        draft.latitude.unwrap_or_default(),
        draft.longitude.unwrap_or_default()
    );
}

fn parse_location_choice(text: Option<&str>, candidate_count: usize) -> Option<usize> {
    text.map(str::trim)
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|choice| (1..=candidate_count).contains(choice))
}

fn format_location_candidates(candidates: &[CityCandidate]) -> String {
    candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| format!("{}. {}", index + 1, candidate.label))
        .collect::<Vec<_>>()
        .join("\n")
}

fn should_keep_existing_location(query: &str, draft: &Profile) -> bool {
    matches!(
        non_empty_value(draft.location.as_deref()),
        Some(location)
            if location == query.trim() && draft.latitude.is_some() && draft.longitude.is_some()
    )
}

fn kept_photo_file_id(text: Option<&str>, language: Language, draft: &Profile) -> Option<String> {
    if text.map(str::trim) == Some(language.text(TextKey::KeepPreviousPhoto)) {
        non_empty_value(draft.photo.as_deref()).map(ToOwned::to_owned)
    } else {
        None
    }
}

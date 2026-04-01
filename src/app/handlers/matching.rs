use crate::app::chat_ui::{
    build_match_notification_text, prompt_for_incoming_like_decision,
    prompt_for_language_selection, prompt_for_swipe_decision, require_sender, send_profile_card,
    show_candidate_profile, show_main_menu, show_settings_menu, transition_to_edit_menu,
    transition_to_main_menu, transition_to_settings_menu,
};
use crate::app::types::{
    AppDialogue, HandlerResult, IncomingLikeDecision, IncomingLikeTargetKind, Language,
    MainMenuAction, REVEAL_SPINNER_TEXT, SettingsAction, State, SwipeDecision, profile_language,
};
use crate::db::profile_repository::{
    ProfileRow, activate_profile, deactivate_profile, get_next_profile_for_user,
    get_profile_by_user_id, save_profile,
};
use crate::db::swipe_repository::{
    count_incoming_like_targets_for_user, did_user_like_me, get_incoming_like_for_user,
    get_pending_incoming_like_target_for_user, mark_match_shown_to_user, save_swipe,
    was_match_shown_to_user,
};
use crate::domain::{CompleteProfile, Profile};
use crate::telegram::i18n::TextKey;
use crate::telegram::keyboards::make_edit_profile_keyboard;
use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestLinkPreviewExt;
use teloxide::types::ParseMode;

pub async fn handle_profile_action(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    (current_user_profile, displayed_profile_user_id, return_to_main_menu): (Profile, i64, bool),
    pool: PgPool,
) -> HandlerResult {
    let language = profile_language(&current_user_profile);

    let Some(text) = msg.text() else {
        prompt_for_swipe_decision(&bot, msg.chat.id, language).await?;
        return Ok(());
    };

    let Some(decision) = SwipeDecision::parse(text) else {
        prompt_for_swipe_decision(&bot, msg.chat.id, language).await?;
        return Ok(());
    };

    let Some(sender_info) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    apply_swipe_decision(
        &bot,
        msg.chat.id,
        sender_info.telegram_user_id,
        displayed_profile_user_id,
        decision,
        language,
        &pool,
    )
    .await?;

    if return_to_main_menu {
        transition_to_main_menu(&bot, &dialogue, msg.chat.id, current_user_profile).await?;
    } else {
        show_next_profile_or_menu(&bot, &dialogue, &msg, current_user_profile, &pool).await?;
    }

    Ok(())
}

pub async fn handle_main_menu(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    profile: Profile,
    pool: PgPool,
) -> HandlerResult {
    let language = profile_language(&profile);

    let Some(text) = msg.text() else {
        show_main_menu(&bot, msg.chat.id, language).await?;
        return Ok(());
    };

    let Some(action) = MainMenuAction::parse(text) else {
        show_main_menu(&bot, msg.chat.id, language).await?;
        return Ok(());
    };

    handle_main_menu_action(&bot, &dialogue, &msg, profile, &pool, action).await?;

    Ok(())
}

pub async fn resume_main_menu_flow(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    profile: Profile,
    pool: PgPool,
) -> HandlerResult {
    if let Some(action) = msg.text().and_then(MainMenuAction::parse) {
        handle_main_menu_action(&bot, &dialogue, &msg, profile, &pool, action).await?;
    } else {
        transition_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
    }

    Ok(())
}

pub async fn handle_settings_menu(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    profile: Profile,
) -> HandlerResult {
    let language = profile_language(&profile);

    let Some(text) = msg.text() else {
        show_settings_menu(&bot, msg.chat.id, language).await?;
        return Ok(());
    };

    match SettingsAction::parse(text) {
        Some(SettingsAction::ChangeLanguage) => {
            prompt_for_language_selection(&bot, msg.chat.id).await?;
            dialogue
                .update(State::WaitingForLanguagePreference { profile })
                .await?;
        }
        Some(SettingsAction::BackToMainMenu) => {
            transition_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
        }
        None => {
            show_settings_menu(&bot, msg.chat.id, language).await?;
        }
    }

    Ok(())
}

pub async fn handle_language_preference_selection(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    mut profile: Profile,
    pool: PgPool,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        prompt_for_language_selection(&bot, msg.chat.id).await?;
        return Ok(());
    };

    let Some(language) = Language::from_text(text) else {
        prompt_for_language_selection(&bot, msg.chat.id).await?;
        return Ok(());
    };

    profile.language_code = Some(language.as_db_code().to_owned());

    let complete_profile = match CompleteProfile::try_from(&profile) {
        Ok(profile) => profile,
        Err(error) => {
            log::warn!(
                "Unable to update language preference for user {:?}: {}",
                profile.telegram_user_id,
                error
            );
            transition_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
            return Ok(());
        }
    };

    save_profile(&pool, &complete_profile).await?;
    transition_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;

    Ok(())
}

pub async fn handle_incoming_like_decision(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    (profile, incoming_like_user_id, pending_like_count, target_kind): (
        Profile,
        i64,
        i64,
        IncomingLikeTargetKind,
    ),
    pool: PgPool,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        prompt_for_incoming_like_decision(&bot, msg.chat.id, &profile, pending_like_count).await?;
        return Ok(());
    };

    match IncomingLikeDecision::parse(text) {
        Some(IncomingLikeDecision::Show) => {
            let Some(incoming_like_profile_row) =
                get_profile_by_user_id(&pool, incoming_like_user_id).await?
            else {
                show_no_new_likes_and_return_to_menu(&bot, &dialogue, msg.chat.id, profile).await?;
                return Ok(());
            };

            match target_kind {
                IncomingLikeTargetKind::IncomingLike => {
                    reveal_profile(
                        &bot,
                        &dialogue,
                        msg.chat.id,
                        profile,
                        incoming_like_profile_row,
                        true,
                        None,
                    )
                    .await?;
                }
                IncomingLikeTargetKind::PendingMutualMatch => {
                    let Some(user_id) = profile.telegram_user_id else {
                        log::warn!("Unable to reveal pending mutual match: missing user id");
                        return Ok(());
                    };

                    reveal_pending_mutual_match(
                        &bot,
                        &dialogue,
                        msg.chat.id,
                        profile,
                        user_id,
                        incoming_like_profile_row,
                        &pool,
                    )
                    .await?;
                }
            }
        }
        Some(IncomingLikeDecision::StopViewing) => {
            let Some(sender_info) = require_sender(&bot, &msg).await? else {
                return Ok(());
            };

            deactivate_and_return_to_menu(
                &bot,
                &dialogue,
                msg.chat.id,
                sender_info.telegram_user_id,
                profile,
                &pool,
            )
            .await?;
        }
        None => {
            prompt_for_incoming_like_decision(&bot, msg.chat.id, &profile, pending_like_count)
                .await?;
        }
    }

    Ok(())
}

pub async fn handle_global_incoming_like_decision(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    pool: PgPool,
) -> HandlerResult {
    let Some(sender_info) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    let Some(current_user_profile_row) =
        get_profile_by_user_id(&pool, sender_info.telegram_user_id).await?
    else {
        prompt_for_language_selection(&bot, msg.chat.id).await?;
        dialogue.update(State::WaitingForLanguage).await?;
        return Ok(());
    };

    let current_user_profile = current_user_profile_row.to_profile();

    let Some(text) = msg.text() else {
        return Ok(());
    };

    let Some(decision) = IncomingLikeDecision::parse(text) else {
        return Ok(());
    };

    match decision {
        IncomingLikeDecision::Show => {
            let Some(target) =
                get_pending_incoming_like_target_for_user(&pool, sender_info.telegram_user_id)
                    .await?
            else {
                show_no_new_likes_and_return_to_menu(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    current_user_profile,
                )
                .await?;
                return Ok(());
            };

            match target.target_kind {
                IncomingLikeTargetKind::IncomingLike => {
                    reveal_profile(
                        &bot,
                        &dialogue,
                        msg.chat.id,
                        current_user_profile,
                        target.profile_row,
                        true,
                        None,
                    )
                    .await?;
                }
                IncomingLikeTargetKind::PendingMutualMatch => {
                    reveal_pending_mutual_match(
                        &bot,
                        &dialogue,
                        msg.chat.id,
                        current_user_profile,
                        sender_info.telegram_user_id,
                        target.profile_row,
                        &pool,
                    )
                    .await?;
                }
            }
        }
        IncomingLikeDecision::StopViewing => {
            deactivate_and_return_to_menu(
                &bot,
                &dialogue,
                msg.chat.id,
                sender_info.telegram_user_id,
                current_user_profile,
                &pool,
            )
            .await?;
        }
    }

    Ok(())
}

async fn apply_swipe_decision(
    bot: &Bot,
    chat_id: ChatId,
    sender_user_id: i64,
    displayed_profile_user_id: i64,
    decision: SwipeDecision,
    language: Language,
    pool: &PgPool,
) -> HandlerResult {
    match decision {
        SwipeDecision::Like => {
            log::info!(
                "User {} liked user {}",
                sender_user_id,
                displayed_profile_user_id
            );
            save_swipe(pool, sender_user_id, displayed_profile_user_id, true).await?;

            if did_user_like_me(pool, sender_user_id, displayed_profile_user_id).await? {
                notify_mutual_match(
                    bot,
                    chat_id,
                    sender_user_id,
                    displayed_profile_user_id,
                    language,
                    pool,
                )
                .await?;
            } else {
                notify_liked_user(bot, displayed_profile_user_id, pool).await?;
            }
        }
        SwipeDecision::Skip => {
            log::info!(
                "User {} skipped user {}",
                sender_user_id,
                displayed_profile_user_id
            );
            save_swipe(pool, sender_user_id, displayed_profile_user_id, false).await?;
        }
    }

    Ok(())
}

async fn deactivate_and_return_to_menu(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    user_id: i64,
    profile: Profile,
    pool: &PgPool,
) -> HandlerResult {
    let language = profile_language(&profile);

    deactivate_profile(pool, user_id).await?;
    bot.send_message(chat_id, language.text(TextKey::InactiveNow))
        .await?;
    transition_to_main_menu(bot, dialogue, chat_id, profile).await?;

    Ok(())
}

async fn notify_liked_user(bot: &Bot, target_user_id: i64, pool: &PgPool) -> HandlerResult {
    let Some(target_profile_row) = get_profile_by_user_id(pool, target_user_id).await? else {
        log::warn!("Unable to notify liked user {target_user_id}: profile not found");
        return Ok(());
    };

    let target_profile = target_profile_row.to_profile();
    let pending_like_count = count_incoming_like_targets_for_user(pool, target_user_id).await?;
    prompt_for_incoming_like_decision(bot, ChatId(target_profile_row.chat_id), &target_profile, pending_like_count)
        .await?;

    Ok(())
}

async fn notify_mutual_match(
    bot: &Bot,
    requester_chat_id: ChatId,
    requester_user_id: i64,
    other_user_id: i64,
    language: Language,
    pool: &PgPool,
) -> HandlerResult {
    let Some(other_profile_row) = get_profile_by_user_id(pool, other_user_id).await? else {
        log::warn!("Unable to finalize match with user {other_user_id}: profile not found");
        return Ok(());
    };

    mark_match_shown_to_user(pool, requester_user_id, other_user_id).await?;

    bot.send_message(
        requester_chat_id,
        build_match_notification_text(&other_profile_row, language),
    )
    .parse_mode(ParseMode::Html)
    .disable_link_preview(true)
    .await?;

    if !was_match_shown_to_user(pool, other_user_id, requester_user_id).await? {
        let other_profile = other_profile_row.to_profile();
        let pending_like_count = count_incoming_like_targets_for_user(pool, other_user_id).await?;
        prompt_for_incoming_like_decision(
            bot,
            ChatId(other_profile_row.chat_id),
            &other_profile,
            pending_like_count,
        )
        .await?;
    }

    Ok(())
}

async fn reveal_profile(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    current_user_profile: Profile,
    target_profile_row: ProfileRow,
    return_to_main_menu: bool,
    mark_as_shown: Option<(&PgPool, i64)>,
) -> HandlerResult {
    let language = profile_language(&current_user_profile);
    let displayed_profile = target_profile_row.to_profile();
    let displayed_profile_user_id = target_profile_row.telegram_user_id;

    if let Some((pool, user_id)) = mark_as_shown {
        mark_match_shown_to_user(pool, user_id, displayed_profile_user_id).await?;
    }

    bot.send_message(chat_id, REVEAL_SPINNER_TEXT).await?;
    show_candidate_profile(
        bot,
        dialogue,
        chat_id,
        current_user_profile,
        &displayed_profile,
        Some(language.text(TextKey::ThisPersonLikedYou)),
        return_to_main_menu,
    )
    .await?;

    Ok(())
}

async fn reveal_pending_mutual_match(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    current_user_profile: Profile,
    user_id: i64,
    target_profile_row: ProfileRow,
    pool: &PgPool,
) -> HandlerResult {
    let language = profile_language(&current_user_profile);
    let displayed_profile_user_id = target_profile_row.telegram_user_id;
    let displayed_profile = target_profile_row.to_profile();

    mark_match_shown_to_user(pool, user_id, displayed_profile_user_id).await?;

    bot.send_message(chat_id, REVEAL_SPINNER_TEXT).await?;
    send_profile_card(
        bot,
        chat_id,
        &displayed_profile,
        Some(language.text(TextKey::ThisPersonLikedYou)),
        None,
    )
    .await?;
    bot.send_message(
        chat_id,
        build_match_notification_text(&target_profile_row, language),
    )
    .parse_mode(ParseMode::Html)
    .disable_link_preview(true)
    .await?;
    transition_to_main_menu(bot, dialogue, chat_id, current_user_profile).await?;

    Ok(())
}

async fn show_next_profile_or_menu(
    bot: &Bot,
    dialogue: &AppDialogue,
    msg: &Message,
    current_user_profile: Profile,
    pool: &PgPool,
) -> HandlerResult {
    let Some(sender_info) = require_sender(bot, msg).await? else {
        return Ok(());
    };

    let language = profile_language(&current_user_profile);
    if let Some(incoming_like_row) =
        get_incoming_like_for_user(pool, sender_info.telegram_user_id).await?
    {
        reveal_profile(
            bot,
            dialogue,
            msg.chat.id,
            current_user_profile,
            incoming_like_row,
            false,
            None,
        )
        .await?;
        return Ok(());
    }

    let Some(next_profile_row) =
        get_next_profile_for_user(pool, sender_info.telegram_user_id).await?
    else {
        bot.send_message(msg.chat.id, language.text(TextKey::NoMoreProfiles))
            .await?;
        transition_to_main_menu(bot, dialogue, msg.chat.id, current_user_profile).await?;
        return Ok(());
    };

    show_candidate_profile(
        bot,
        dialogue,
        msg.chat.id,
        current_user_profile,
        &next_profile_row.to_profile(),
        None,
        false,
    )
    .await?;

    Ok(())
}

async fn show_no_new_likes_and_return_to_menu(
    bot: &Bot,
    dialogue: &AppDialogue,
    chat_id: ChatId,
    profile: Profile,
) -> HandlerResult {
    let language = profile_language(&profile);

    bot.send_message(chat_id, language.text(TextKey::NoNewLikes))
        .await?;
    transition_to_main_menu(bot, dialogue, chat_id, profile).await?;

    Ok(())
}

async fn handle_main_menu_action(
    bot: &Bot,
    dialogue: &AppDialogue,
    msg: &Message,
    profile: Profile,
    pool: &PgPool,
    action: MainMenuAction,
) -> HandlerResult {
    match action {
        MainMenuAction::ViewProfiles => {
            let Some(sender_info) = require_sender(bot, msg).await? else {
                return Ok(());
            };

            activate_profile(pool, sender_info.telegram_user_id).await?;
            show_next_profile_or_menu(bot, dialogue, msg, profile, pool).await?;
        }
        MainMenuAction::MyProfile => {
            let language = profile_language(&profile);
            send_profile_card(
                bot,
                msg.chat.id,
                &profile,
                None,
                Some(make_edit_profile_keyboard(language).into()),
            )
            .await?;
            transition_to_edit_menu(dialogue, profile).await?;
        }
        MainMenuAction::Settings => {
            transition_to_settings_menu(bot, dialogue, msg.chat.id, profile).await?;
        }
        MainMenuAction::DeactivateProfile => {
            let Some(sender_info) = require_sender(bot, msg).await? else {
                return Ok(());
            };

            deactivate_and_return_to_menu(
                bot,
                dialogue,
                msg.chat.id,
                sender_info.telegram_user_id,
                profile,
                pool,
            )
            .await?;
        }
    }

    Ok(())
}

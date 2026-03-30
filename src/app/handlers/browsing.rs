use crate::app::types::{
    HandlerResult, IncomingLikeDecision, Lang, MainMenuAction, MyDialogue, REVEAL_SPINNER_TEXT,
    SettingsAction, State, SwipeDecision, profile_lang,
};
use crate::app::ui::{
    match_notification_text, move_to_edit_menu, move_to_main_menu, move_to_settings_menu,
    prompt_for_profile_action, prompt_incoming_like_decision, prompt_language_selection,
    require_sender, send_profile, show_main_menu, show_profile_for_action, show_settings_menu,
};
use crate::bot::i18n::TextKey;
use crate::db::profile_repository::{
    ProfileRow, activate_profile, deactivate_profile, get_next_profile_for_user,
    get_profile_by_user_id, save_profile,
};
use crate::db::swipe_repository::{
    did_user_like_me, get_incoming_like_for_user, get_pending_mutual_match_for_user,
    mark_match_shown_to_user, save_swipe, was_match_shown_to_user,
};
use crate::models::{CompleteProfile, Profile};
use sqlx::PgPool;
use teloxide::prelude::*;
use teloxide::sugar::request::RequestLinkPreviewExt;
use teloxide::types::ParseMode;

pub async fn handle_profile_action(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (profile, shown_profile_user_id, return_to_menu): (Profile, i64, bool),
    pool: PgPool,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    let Some(text) = msg.text() else {
        prompt_for_profile_action(&bot, msg.chat.id, lang).await?;
        return Ok(());
    };

    let Some(decision) = SwipeDecision::parse(text) else {
        prompt_for_profile_action(&bot, msg.chat.id, lang).await?;
        return Ok(());
    };

    let Some(sender) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    apply_swipe_decision(
        &bot,
        msg.chat.id,
        sender.telegram_user_id,
        shown_profile_user_id,
        decision,
        lang,
        &pool,
    )
    .await?;

    if return_to_menu {
        move_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
    } else {
        show_next_profile_or_menu(&bot, &dialogue, &msg, profile, &pool).await?;
    }

    Ok(())
}

pub async fn main_menu(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    profile: Profile,
    pool: PgPool,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    let Some(text) = msg.text() else {
        show_main_menu(&bot, msg.chat.id, lang).await?;
        return Ok(());
    };

    let Some(action) = MainMenuAction::parse(text) else {
        show_main_menu(&bot, msg.chat.id, lang).await?;
        return Ok(());
    };

    handle_main_menu_action(&bot, &dialogue, &msg, profile, &pool, action).await?;

    Ok(())
}

pub async fn resume_main_menu(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    profile: Profile,
    pool: PgPool,
) -> HandlerResult {
    if let Some(action) = msg.text().and_then(MainMenuAction::parse) {
        handle_main_menu_action(&bot, &dialogue, &msg, profile, &pool, action).await?;
    } else {
        move_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
    }

    Ok(())
}

pub async fn settings_menu(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    profile: Profile,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    let Some(text) = msg.text() else {
        show_settings_menu(&bot, msg.chat.id, lang).await?;
        return Ok(());
    };

    match SettingsAction::parse(text) {
        Some(SettingsAction::ChangeLanguage) => {
            prompt_language_selection(&bot, msg.chat.id).await?;
            dialogue
                .update(State::WaitingForLanguagePreference { profile })
                .await?;
        }
        Some(SettingsAction::BackToMainMenu) => {
            move_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
        }
        None => {
            show_settings_menu(&bot, msg.chat.id, lang).await?;
        }
    }

    Ok(())
}

pub async fn receive_language_preference(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
    pool: PgPool,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        prompt_language_selection(&bot, msg.chat.id).await?;
        return Ok(());
    };

    let Some(lang) = Lang::from_text(text) else {
        prompt_language_selection(&bot, msg.chat.id).await?;
        return Ok(());
    };

    profile.language_code = Some(lang.as_db_code().to_owned());

    let complete_profile = match CompleteProfile::try_from(&profile) {
        Ok(profile) => profile,
        Err(error) => {
            log::warn!(
                "Unable to update language preference for user {:?}: {}",
                profile.telegram_user_id,
                error
            );
            move_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;
            return Ok(());
        }
    };

    save_profile(&pool, &complete_profile).await?;
    move_to_main_menu(&bot, &dialogue, msg.chat.id, profile).await?;

    Ok(())
}

pub async fn handle_incoming_like_decision(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    (profile, liked_you_user_id): (Profile, i64),
    pool: PgPool,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    let Some(text) = msg.text() else {
        prompt_incoming_like_decision(&bot, msg.chat.id, lang).await?;
        return Ok(());
    };

    match IncomingLikeDecision::parse(text) {
        Some(IncomingLikeDecision::Show) => {
            let Some(liked_you_row) = get_profile_by_user_id(&pool, liked_you_user_id).await?
            else {
                show_no_new_likes_and_return_to_menu(&bot, &dialogue, msg.chat.id, profile).await?;
                return Ok(());
            };

            reveal_profile(
                &bot,
                &dialogue,
                msg.chat.id,
                profile,
                liked_you_row,
                true,
                None,
            )
            .await?;
        }
        Some(IncomingLikeDecision::StopViewing) => {
            let Some(sender) = require_sender(&bot, &msg).await? else {
                return Ok(());
            };

            deactivate_and_return_to_menu(
                &bot,
                &dialogue,
                msg.chat.id,
                sender.telegram_user_id,
                profile,
                &pool,
            )
            .await?;
        }
        None => {
            prompt_incoming_like_decision(&bot, msg.chat.id, lang).await?;
        }
    }

    Ok(())
}

pub async fn global_incoming_like_decision(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    pool: PgPool,
) -> HandlerResult {
    let Some(sender) = require_sender(&bot, &msg).await? else {
        return Ok(());
    };

    let Some(my_profile_row) = get_profile_by_user_id(&pool, sender.telegram_user_id).await? else {
        prompt_language_selection(&bot, msg.chat.id).await?;
        dialogue.update(State::WaitingForLanguage).await?;
        return Ok(());
    };

    let my_profile = my_profile_row.to_profile();

    let Some(text) = msg.text() else {
        return Ok(());
    };

    let Some(decision) = IncomingLikeDecision::parse(text) else {
        return Ok(());
    };

    match decision {
        IncomingLikeDecision::Show => {
            let Some(target) = resolve_incoming_like_target(&pool, sender.telegram_user_id).await?
            else {
                show_no_new_likes_and_return_to_menu(&bot, &dialogue, msg.chat.id, my_profile)
                    .await?;
                return Ok(());
            };

            match target {
                IncomingLikeTarget::IncomingLike(target_row) => {
                    reveal_profile(
                        &bot,
                        &dialogue,
                        msg.chat.id,
                        my_profile,
                        target_row,
                        true,
                        None,
                    )
                    .await?;
                }
                IncomingLikeTarget::PendingMutualMatch(target_row) => {
                    reveal_pending_mutual_match(
                        &bot,
                        &dialogue,
                        msg.chat.id,
                        my_profile,
                        sender.telegram_user_id,
                        target_row,
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
                sender.telegram_user_id,
                my_profile,
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
    shown_profile_user_id: i64,
    decision: SwipeDecision,
    lang: Lang,
    pool: &PgPool,
) -> HandlerResult {
    match decision {
        SwipeDecision::Like => {
            log::info!(
                "User {} liked user {}",
                sender_user_id,
                shown_profile_user_id
            );
            save_swipe(pool, sender_user_id, shown_profile_user_id, true).await?;

            if did_user_like_me(pool, sender_user_id, shown_profile_user_id).await? {
                notify_mutual_match(
                    bot,
                    chat_id,
                    sender_user_id,
                    shown_profile_user_id,
                    lang,
                    pool,
                )
                .await?;
            } else {
                notify_like_target(bot, shown_profile_user_id, pool).await?;
            }
        }
        SwipeDecision::Skip => {
            log::info!(
                "User {} skipped user {}",
                sender_user_id,
                shown_profile_user_id
            );
            save_swipe(pool, sender_user_id, shown_profile_user_id, false).await?;
        }
    }

    Ok(())
}

async fn deactivate_and_return_to_menu(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    user_id: i64,
    profile: Profile,
    pool: &PgPool,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    deactivate_profile(pool, user_id).await?;
    bot.send_message(chat_id, lang.text(TextKey::InactiveNow))
        .await?;
    move_to_main_menu(bot, dialogue, chat_id, profile).await?;

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
    prompt_incoming_like_decision(bot, ChatId(chat_id), lang).await?;

    Ok(())
}

async fn notify_mutual_match(
    bot: &Bot,
    requester_chat_id: ChatId,
    requester_user_id: i64,
    other_user_id: i64,
    lang: Lang,
    pool: &PgPool,
) -> HandlerResult {
    let Some(other_row) = get_profile_by_user_id(pool, other_user_id).await? else {
        log::warn!("Unable to finalize match with user {other_user_id}: profile not found");
        return Ok(());
    };

    mark_match_shown_to_user(pool, requester_user_id, other_user_id).await?;

    bot.send_message(requester_chat_id, match_notification_text(&other_row, lang))
        .parse_mode(ParseMode::Html)
        .disable_link_preview(true)
        .await?;

    let Some(other_chat_id_raw) = other_row.chat_id else {
        log::warn!("Unable to notify matched user {other_user_id}: chat_id is missing");
        return Ok(());
    };

    if !was_match_shown_to_user(pool, other_user_id, requester_user_id).await? {
        prompt_incoming_like_decision(
            bot,
            ChatId(other_chat_id_raw),
            Lang::from_db_code(&other_row.language_code),
        )
        .await?;
    }

    Ok(())
}

async fn reveal_profile(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    my_profile: Profile,
    target_row: ProfileRow,
    return_to_menu: bool,
    mark_as_shown: Option<(&PgPool, i64)>,
) -> HandlerResult {
    let lang = profile_lang(&my_profile);
    let shown_profile_user_id = target_row.telegram_user_id;
    let shown_profile = target_row.to_profile();

    if let Some((pool, user_id)) = mark_as_shown {
        mark_match_shown_to_user(pool, user_id, shown_profile_user_id).await?;
    }

    bot.send_message(chat_id, REVEAL_SPINNER_TEXT).await?;
    show_profile_for_action(
        bot,
        dialogue,
        chat_id,
        my_profile,
        shown_profile_user_id,
        &shown_profile,
        Some(lang.text(TextKey::ThisPersonLikedYou)),
        None,
        return_to_menu,
    )
    .await?;

    Ok(())
}

async fn reveal_pending_mutual_match(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    my_profile: Profile,
    user_id: i64,
    target_row: ProfileRow,
    pool: &PgPool,
) -> HandlerResult {
    let lang = profile_lang(&my_profile);
    let shown_profile_user_id = target_row.telegram_user_id;
    let shown_profile = target_row.to_profile();

    mark_match_shown_to_user(pool, user_id, shown_profile_user_id).await?;

    bot.send_message(chat_id, REVEAL_SPINNER_TEXT).await?;
    send_profile(
        bot,
        chat_id,
        &shown_profile,
        Some(lang.text(TextKey::ThisPersonLikedYou)),
        None,
        None,
    )
    .await?;
    bot.send_message(chat_id, match_notification_text(&target_row, lang))
        .parse_mode(ParseMode::Html)
        .disable_link_preview(true)
        .await?;
    move_to_main_menu(bot, dialogue, chat_id, my_profile).await?;

    Ok(())
}

async fn resolve_incoming_like_target(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<IncomingLikeTarget>, sqlx::Error> {
    if let Some(incoming_like_row) = get_incoming_like_for_user(pool, user_id).await? {
        Ok(Some(IncomingLikeTarget::IncomingLike(incoming_like_row)))
    } else if let Some(mutual_match_row) = get_pending_mutual_match_for_user(pool, user_id).await? {
        Ok(Some(IncomingLikeTarget::PendingMutualMatch(
            mutual_match_row,
        )))
    } else {
        Ok(None)
    }
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
        reveal_profile(
            bot,
            dialogue,
            msg.chat.id,
            my_profile,
            incoming_like_row,
            false,
            None,
        )
        .await?;
        return Ok(());
    }

    let Some(next_profile_row) = get_next_profile_for_user(pool, sender.telegram_user_id).await?
    else {
        bot.send_message(msg.chat.id, lang.text(TextKey::NoMoreProfiles))
            .await?;
        move_to_main_menu(bot, dialogue, msg.chat.id, my_profile).await?;
        return Ok(());
    };

    show_profile_for_action(
        bot,
        dialogue,
        msg.chat.id,
        my_profile,
        next_profile_row.telegram_user_id,
        &next_profile_row.to_profile(),
        None,
        None,
        false,
    )
    .await?;

    Ok(())
}

async fn show_no_new_likes_and_return_to_menu(
    bot: &Bot,
    dialogue: &MyDialogue,
    chat_id: ChatId,
    profile: Profile,
) -> HandlerResult {
    let lang = profile_lang(&profile);

    bot.send_message(chat_id, lang.text(TextKey::NoNewLikes))
        .await?;
    move_to_main_menu(bot, dialogue, chat_id, profile).await?;

    Ok(())
}

enum IncomingLikeTarget {
    IncomingLike(ProfileRow),
    PendingMutualMatch(ProfileRow),
}

async fn handle_main_menu_action(
    bot: &Bot,
    dialogue: &MyDialogue,
    msg: &Message,
    profile: Profile,
    pool: &PgPool,
    action: MainMenuAction,
) -> HandlerResult {
    match action {
        MainMenuAction::ViewProfiles => {
            let Some(sender) = require_sender(bot, msg).await? else {
                return Ok(());
            };

            activate_profile(pool, sender.telegram_user_id).await?;
            show_next_profile_or_menu(bot, dialogue, msg, profile, pool).await?;
        }
        MainMenuAction::MyProfile => {
            send_profile(bot, msg.chat.id, &profile, None, None, None).await?;
            move_to_edit_menu(bot, dialogue, msg.chat.id, profile).await?;
        }
        MainMenuAction::Settings => {
            move_to_settings_menu(bot, dialogue, msg.chat.id, profile).await?;
        }
        MainMenuAction::DeactivateProfile => {
            let Some(sender) = require_sender(bot, msg).await? else {
                return Ok(());
            };

            deactivate_and_return_to_menu(
                bot,
                dialogue,
                msg.chat.id,
                sender.telegram_user_id,
                profile,
                pool,
            )
            .await?;
        }
    }

    Ok(())
}

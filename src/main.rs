mod bot;
mod db;
mod models;

use crate::bot::keyboards::{make_edit_profile_keyboard, make_gender_keyboard, make_profile_action_keyboard, make_profile_confirmation_keyboard, make_start_keyboard};
use crate::db::connect_db;
use crate::db::profile_repository::{get_next_profile_for_user, save_profile};

use crate::models::{Gender, Profile};
use sqlx::PgPool;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
};
use teloxide::types::{FileId, ReplyMarkup};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
enum State {
    #[default]
    Start,
    WaitingForName,
    WaitingForGender {
        profile: Profile,
    },
    WaitingForLookingFor {
        profile: Profile,
    },
    WaitingForAge {
        profile: Profile,
    },
    WaitingForLocation {
        profile: Profile,
    },
    WaitingForDescription {
        profile: Profile,
    },
    WaitingForDescriptionOnlyEditing {
        profile: Profile,
    },
    WaitingForPhoto {
        profile: Profile,
    },
    WaitingForPhotoOnlyEditing{
        profile: Profile,
    },
    ProfileCreatedCorrectOrNo {
        profile: Profile,
    },
    EditProfileMenu {
        profile: Profile,
    },
    ShowingOtherProfiles {
        shown_profile_user_id: i64,
    },
}

async fn send_profile(
    bot: &Bot,
    chat_id: ChatId,
    profile: &Profile,
    footer_text: Option<&str>,
    reply_markup: Option<ReplyMarkup>,
) -> HandlerResult {
    let text = profile.display_text();

    if let Some(photo) = profile.photo.clone() {
        let mut request = bot.send_photo(
            chat_id,
            teloxide::types::InputFile::file_id(FileId::from(photo)),
        )
            .caption(text);

        if let Some(markup) = reply_markup {
            request = request.reply_markup(markup);
        }

        request.await?;
    } else {
        let mut message_text = text;

        if let Some(footer) = footer_text {
            message_text.push_str("\n\n");
            message_text.push_str(footer);
        }

        let mut request = bot.send_message(chat_id, message_text);

        if let Some(markup) = reply_markup {
            request = request.reply_markup(markup);
        }

        request.await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool: PgPool = connect_db(&database_url)
        .await
        .expect("Failed to connect to database");

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch( // vzdy sa prednostne pozera, ci nie je /start
                dptree::filter(|msg: Message| {
                    msg.text().map(|t| t.starts_with("/start")).unwrap_or(false)
                })
                    .endpoint(global_start),
            )
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::WaitingForName].endpoint(receive_name))
            .branch(dptree::case![State::WaitingForGender { profile }].endpoint(receive_gender))
            .branch(
                dptree::case![State::WaitingForLookingFor { profile }]
                    .endpoint(receive_looking_for),
            )
            .branch(dptree::case![State::WaitingForAge { profile }].endpoint(receive_age))
            .branch(dptree::case![State::WaitingForLocation { profile }].endpoint(receive_location))
            .branch(
                dptree::case![State::WaitingForDescription { profile }]
                    .endpoint(receive_description),
            )
            .branch(dptree::case![State::WaitingForPhoto { profile }].endpoint(receive_photo))
            .branch(
                dptree::case![State::ProfileCreatedCorrectOrNo { profile }].endpoint(correct_or_no),
            )
            .branch(dptree::case![State::EditProfileMenu { profile }].endpoint(edit_profile_menu))
            .branch(dptree::case![State::WaitingForPhotoOnlyEditing{ profile }].endpoint(edit_photo))
            .branch(dptree::case![State::WaitingForDescriptionOnlyEditing{ profile }].endpoint(edit_description))
            .branch(
                dptree::case![State::ShowingOtherProfiles { shown_profile_user_id }]
                    .endpoint(handle_profile_action),
            )
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn global_start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        if text == "/start" {
            bot.send_message(
                msg.chat.id,
                "Ahoj! Vitaj v TeleMatch.\nTu si môžeš vytvoriť profil, prezerať ostatných používateľov a získavať zhody.\nPre zoznam príkazov použi /help.",
            )
                .reply_markup(make_start_keyboard())
                .await?;

            dialogue.update(State::Start).await?;
        }
    }

    Ok(())
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        match text {
            "Vytvoriť profil" => {
                bot.send_message(msg.chat.id, "What's your name?")
                    .await?;
                dialogue.update(State::WaitingForName).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn receive_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "What's your name?").await?;
        return Ok(());
    };

    bot.send_message(msg.chat.id, "Specify your gender")
        .reply_markup(make_gender_keyboard())
        .await?;

    let mut profile = Profile::default();
    profile.name = Some(text.to_string());
    dialogue.update(State::WaitingForGender { profile }).await?;
    Ok(())
}

async fn receive_gender(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Specify your gender")
            .reply_markup(make_gender_keyboard())
            .await?;
        return Ok(());
    };

    let Some(gender) = Gender::from_text(text) else {
        bot.send_message(msg.chat.id, "Specify your gender")
            .reply_markup(make_gender_keyboard())
            .await?;
        return Ok(());
    };

    bot.send_message(msg.chat.id, "Who are you looking for?")
        .reply_markup(make_gender_keyboard())
        .await?;

    profile.gender = Some(gender);
    dialogue
        .update(State::WaitingForLookingFor { profile })
        .await?;

    Ok(())
}

async fn receive_looking_for(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Who are you looking for?")
            .reply_markup(make_gender_keyboard())
            .await?;
        return Ok(());
    };

    let Some(looking_for) = Gender::from_text(text) else {
        bot.send_message(msg.chat.id, "Who are you looking for?")
            .reply_markup(make_gender_keyboard())
            .await?;
        return Ok(());
    };

    bot.send_message(msg.chat.id, "Your age?").await?;

    profile.looking_for = Some(looking_for);
    dialogue.update(State::WaitingForAge { profile }).await?;
    Ok(())
}

async fn receive_age(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Please, enter your age as number")
            .await?;
        return Ok(());
    };

    let age = match text.parse::<u8>() {
        Ok(age) if age <= 100 => age,
        _ => {
            bot.send_message(msg.chat.id, "Age cannot be more than 100.")
                .await?;
            return Ok(());
        }
    };

    bot.send_message(msg.chat.id, "What city are you from?")
        .await?;

    profile.age = Some(age);
    dialogue
        .update(State::WaitingForLocation { profile })
        .await?;
    Ok(())
}

async fn receive_location(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "What city are you from?")
            .await?;
        return Ok(());
    };

    let location = text.to_string();

    bot.send_message(msg.chat.id, "Tell more about yourself")
        .await?;
    // TODO add skip for description

    profile.location = Some(location);
    dialogue
        .update(State::WaitingForDescription { profile })
        .await?;

    Ok(())
}

async fn receive_description(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Tell more about yourself")
            .await?;
        return Ok(());
    };

    let description = text.to_string();

    bot.send_message(msg.chat.id, "Send your photo").await?;

    profile.description = Some(description);
    dialogue.update(State::WaitingForPhoto { profile }).await?;

    Ok(())
}

async fn receive_photo(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
) -> HandlerResult {
    let Some(photo_sizes) = msg.photo() else {
        bot.send_message(msg.chat.id, "Send your photo").await?;
        return Ok(());
    };

    let Some(largest_photo) = photo_sizes.last() else {
        bot.send_message(msg.chat.id, "Send your photo again")
            .await?;
        return Ok(());
    };

    let photo = largest_photo.file.id.clone();
    profile.photo = Some(photo.to_string());

    send_profile(&bot, msg.chat.id, &profile, None, None).await?;

    bot.send_message(msg.chat.id, "Correct?")
        .reply_markup(make_profile_confirmation_keyboard())
        .await?;
    dialogue
        .update(State::ProfileCreatedCorrectOrNo { profile })
        .await?;

    Ok(())
}

async fn correct_or_no(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    profile: Profile,
    pool: PgPool,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Please, use the keyboard below.")
            .reply_markup(make_profile_confirmation_keyboard())
            .await?;
        return Ok(());
    };

    match text.trim() {
        "Yes" => {
            let telegram_user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
            save_profile(&pool, telegram_user_id, &profile).await?;
            log::info!("User profile for user {telegram_user_id} was updated in DB");

            bot.send_message(msg.chat.id, "✨🔍").await?;
            // TODO here start showing other profiles

            dialogue.update(State::ShowingOtherProfiles{ shown_profile_user_id: 0 }).await?;
        }
        "Edit my profile" => {
            bot.send_message(
                msg.chat.id,
                "1. View profiles\n2. Edit my profile\n3. Change my photo\n4. Change profile text",
            )
            .reply_markup(make_edit_profile_keyboard())
            .await?;

            dialogue.update(State::EditProfileMenu { profile }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please choose one of the available options.")
                .reply_markup(make_profile_confirmation_keyboard())
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
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Please choose one of the available options.")
            .await?;

        return Ok(());
    };

    match text {
        "1" => {
            bot.send_message(msg.chat.id, "✨🔍").await?;
            // TODO here start showing other profiles
        }
        "2" => {
            bot.send_message(msg.chat.id, "What's your name?").await?;
            dialogue.update(State::WaitingForName).await?;
        }
        "3" => {
            // change my photo
            bot.send_message(msg.chat.id, "Send your photo").await?;
            dialogue.update(State::WaitingForPhotoOnlyEditing{ profile }).await?;
        }
        "4" => {
            // change profile text
            bot.send_message(msg.chat.id, "Tell more about yourself").await?;
            dialogue.update(State::WaitingForDescriptionOnlyEditing{ profile }).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn edit_photo(bot: Bot, dialogue: MyDialogue, msg: Message, mut profile: Profile, pool: PgPool) -> HandlerResult {
    let Some(photo_sizes) = msg.photo() else {
        bot.send_message(msg.chat.id, "Send your photo").await?;
        return Ok(());
    };

    let Some(largest_photo) = photo_sizes.last() else {
        bot.send_message(msg.chat.id, "Send your photo again")
            .await?;
        return Ok(());
    };

    let photo = largest_photo.file.id.clone();
    profile.photo = Some(photo.to_string());

    let telegram_user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    save_profile(&pool, telegram_user_id, &profile).await?;
    log::info!("Photo for user {telegram_user_id} was updated in DB");

    send_profile(&bot, msg.chat.id, &profile, None, None).await?;

    bot.send_message(msg.chat.id, "Correct?")
        .reply_markup(make_profile_confirmation_keyboard())
        .await?;
    dialogue
        .update(State::ProfileCreatedCorrectOrNo { profile })
        .await?;

    Ok(())
}

async fn edit_description(bot: Bot, dialogue: MyDialogue, msg: Message, mut profile: Profile, pool: PgPool) -> HandlerResult{
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let description = text.trim();
    profile.description = Some(description.to_string());
    let telegram_user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    save_profile(&pool, telegram_user_id, &profile).await?;
    log::info!("Description for user {telegram_user_id} was updated in DB");


    send_profile(&bot, msg.chat.id, &profile, None, None).await?;

    bot.send_message(msg.chat.id, "Correct?")
        .reply_markup(make_profile_confirmation_keyboard())
        .await?;
    dialogue
        .update(State::ProfileCreatedCorrectOrNo { profile })
        .await?;

    Ok(())
}

async fn handle_profile_action(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    shown_profile_user_id: i64,
    pool: PgPool,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Please use ❤️ or 👎.")
            .reply_markup(make_profile_action_keyboard())
            .await?;
        return Ok(());
    };

    let telegram_user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    match text.trim() {
        "❤️" => {
            log::info!("User {telegram_user_id} liked user {shown_profile_user_id}");
            // TODO save like to DB
        }
        "👎" => {
            log::info!("User {telegram_user_id} disliked user {shown_profile_user_id}");
            // TODO save dislike or just skip
        }
        _ => {
            bot.send_message(msg.chat.id, "Please use ❤️ or 👎.")
                .reply_markup(make_profile_action_keyboard())
                .await?;
            return Ok(());
        }
    }

    let next_profile_row = get_next_profile_for_user(&pool, telegram_user_id).await?;

    let Some(next_profile_row) = next_profile_row else {
        bot.send_message(msg.chat.id, "No more matching profiles found.")
            .reply_markup(make_start_keyboard())
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    };

    let next_shown_profile_user_id = next_profile_row.telegram_user_id;
    let next_profile = next_profile_row.into_profile();

    send_profile(&bot, msg.chat.id, &next_profile, None, Some(make_profile_action_keyboard().into())).await?;

    dialogue
        .update(State::ShowingOtherProfiles {
            shown_profile_user_id: next_shown_profile_user_id,
        })
        .await?;

    Ok(())
}


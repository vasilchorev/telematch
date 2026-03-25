mod bot;
use crate::bot::keyboards::{make_gender_keyboard, make_start_keyboard};

use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
    types::{KeyboardButton, KeyboardMarkup},
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
struct Profile {
    name: Option<String>,
    gender: Option<Gender>,
    looking_for: Option<Gender>,
    age: Option<u8>,
    location: Option<String>,
    description: Option<String>,
    photo: Option<String>,
}

#[derive(Clone)]
enum Gender {
    Male,
    Female,
}

impl Gender {
    fn from_text(text: &str) -> Option<Self> {
        match text.trim() {
            "Muž" => Some(Self::Male),
            "Žena" => Some(Self::Female),
            _ => None,
        }
    }

    fn as_text(&self) -> &'static str {
        match self {
            Self::Male => "Muž",
            Self::Female => "Žena",
        }
    }
}

#[derive(Clone, Default)]
enum State {
    #[default]
    Start,
    WaitingForName,
    WaitingForGender {
        profile: Profile
    },
    WaitingForLookingFor {
        profile: Profile
    },
    WaitingForAge {
        profile: Profile
    },
    WaitingForLocation {
        profile: Profile
    },
    WaitingForDescription {
        profile: Profile
    },
    WaitingForPhoto {
        profile: Profile
    },
    ProfileCreated {
        profile: Profile
    },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::WaitingForName].endpoint(receive_name))
            .branch(dptree::case![State::WaitingForGender { profile }].endpoint(receive_gender))
            .branch(dptree::case![State::WaitingForLookingFor { profile }].endpoint(receive_looking_for))
            .branch(
                dptree::case![State::WaitingForAge {
                    profile
                }]
                    .endpoint(receive_age),
            )
            .branch(
                dptree::case![State::WaitingForLocation {
                    profile
                }]
                    .endpoint(receive_location),
            )
            .branch(
                dptree::case![State::WaitingForDescription {
                    profile
                }]
                    .endpoint(receive_description),
            )
            .branch(
                dptree::case![State::WaitingForPhoto {
                profile
            }]
                    .endpoint(receive_photo),
            )
    )
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        match text {
            "/start" => {
                bot.send_message(
                    msg.chat.id,
                    "Ahoj! Vitaj v TeleMatch.\nTu si môžeš vytvoriť profil, prezerať ostatných používateľov a získavať zhody.\nPre zoznam príkazov použi /help.",
                )
                    .reply_markup(make_start_keyboard())
                    .await?;
            }
            "Vytvoriť profil" => {
                bot.send_message(msg.chat.id, "Zadaj svoje meno.").await?;
                dialogue.update(State::WaitingForName).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn receive_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Prosím, pošli meno ako text.")
            .await?;
        return Ok(());
    };


    bot.send_message(msg.chat.id, "Vyber svoje pohlavie:")
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
        bot.send_message(msg.chat.id, "Vyber pohlavie pomocou tlačidiel.")
            .reply_markup(make_gender_keyboard())
            .await?;
        return Ok(());
    };

    let Some(gender) = Gender::from_text(text) else {
        bot.send_message(msg.chat.id, "Prosím, vyber Muž alebo Žena.")
            .reply_markup(make_gender_keyboard())
            .await?;
        return Ok(());
    };

    bot.send_message(msg.chat.id, "Koho chceš vidieť?")
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
        bot.send_message(msg.chat.id, "Vyber možnosť pomocou tlačidiel.")
            .reply_markup(make_gender_keyboard())
            .await?;
        return Ok(());
    };

    let Some(looking_for) = Gender::from_text(text) else {
        bot.send_message(msg.chat.id, "Prosím, vyber jednu z možností: Muž alebo Žena.")
            .reply_markup(make_gender_keyboard())
            .await?;
        return Ok(());
    };

    bot.send_message(msg.chat.id, "Koľko máš rokov?").await?;

    profile.looking_for = Some(looking_for);
    dialogue
        .update(State::WaitingForAge {
            profile
        })
        .await?;
    Ok(())
}

async fn receive_age(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Prosím, zadaj vek ako číslo.")
            .await?;
        return Ok(());
    };

    let age = match text.parse::<u8>() {
        Ok(age) if age >= 18 && age <= 100 => age,
        _ => {
            bot.send_message(msg.chat.id, "Prosím, zadaj platný vek ako číslo.")
                .await?;
            return Ok(());
        }
    };

    bot.send_message(msg.chat.id, "Odkiaľ si?").await?;

    profile.age = Some(age);
    dialogue
        .update(State::WaitingForLocation {
           profile
        })
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
        bot.send_message(msg.chat.id, "Prosím, pošli lokalitu ako text.")
            .await?;
        return Ok(());
    };

    let location = text.to_string();

    bot.send_message(msg.chat.id, "Napíš krátky popis o sebe.")
        .await?;

    profile.location = Some(location);
    dialogue
        .update(State::WaitingForDescription {
            profile
        })
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
        bot.send_message(msg.chat.id, "Prosím, pošli popis ako text.")
            .await?;
        return Ok(());
    };

    let description = text.to_string();

    bot.send_message(msg.chat.id, "Teraz pošli svoju profilovú fotku.")
        .await?;

    profile.description = Some(description);
    dialogue
        .update(State::WaitingForPhoto {
           profile
        })
        .await?;

    Ok(())
}

async fn receive_photo(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    mut profile: Profile,
) -> HandlerResult {
    let Some(photo_sizes) = msg.photo() else {
        bot.send_message(msg.chat.id, "Prosím, pošli jednu fotku.")
            .await?;
        return Ok(());
    };

    let Some(largest_photo) = photo_sizes.last() else {
        bot.send_message(msg.chat.id, "Nepodarilo sa načítať fotku, skús to znova.")
            .await?;
        return Ok(());
    };

    let photo = largest_photo.file.id.to_string();
    profile.photo = Some(photo);

    bot.send_message(
        msg.chat.id,
        format!(
            "Profil bol úspešne vytvorený:\n\nMeno: {:?}\nPohlavie: {}\nChcem vidieť: {}\nVek: {:?}\nLokalita: {:?}\nPopis: {:?}",
            profile.name,
            profile.gender.clone().unwrap().as_text(),
            profile.looking_for.clone().unwrap().as_text(),
            profile.age,
            profile.location,
            profile.description,
        ),
    )
        .await?;

    dialogue
        .update(State::ProfileCreated { profile })
        .await?;

    Ok(())
}
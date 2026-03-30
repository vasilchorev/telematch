pub mod handlers;
pub mod types;
pub mod ui;

pub use types::Lang;

use crate::app::handlers::{confirm_profile, edit_description, edit_photo, edit_profile_menu, global_incoming_like_decision, global_start, handle_incoming_like_decision, handle_profile_action, main_menu, receive_age, receive_description, receive_gender, receive_language, receive_language_preference, receive_location, receive_location_choice, receive_looking_for, receive_name, receive_photo, settings_menu, start};
use crate::app::types::State;
use crate::app::ui::{is_incoming_like_decision_message, is_start_command};
use crate::db::connect_db;
use sqlx::PgPool;
use teloxide::{dispatching::dialogue::InMemStorage, dptree, prelude::*};

pub async fn run() {
    dotenvy::dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting TeleMatch bot");

    let bot = Bot::from_env();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let geocoder = crate::geocoding::Geocoder::new()
        .expect("GOOGLE_MAPS_API_KEY must be set and geocoder must initialize");

    let pool: PgPool = connect_db(&database_url)
        .await
        .expect("Failed to connect to database");

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::filter(is_start_command).endpoint(global_start))
            .branch(
                dptree::filter(is_incoming_like_decision_message)
                    .endpoint(global_incoming_like_decision),
            )
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::WaitingForLanguage].endpoint(receive_language))
            .branch(
                dptree::case![State::WaitingForLanguagePreference { profile }]
                    .endpoint(receive_language_preference),
            )
            .branch(dptree::case![State::WaitingForName { draft }].endpoint(receive_name))
            .branch(dptree::case![State::WaitingForGender { draft }].endpoint(receive_gender))
            .branch(
                dptree::case![State::WaitingForLookingFor { draft }].endpoint(receive_looking_for),
            )
            .branch(dptree::case![State::WaitingForAge { draft }].endpoint(receive_age))
            .branch(dptree::case![State::WaitingForLocation { draft }].endpoint(receive_location))
            .branch(
                dptree::case![State::WaitingForLocationChoice { draft, candidates }]
                    .endpoint(receive_location_choice),
            )
            .branch(
                dptree::case![State::WaitingForDescription { draft }].endpoint(receive_description),
            )
            .branch(dptree::case![State::WaitingForPhoto { draft }].endpoint(receive_photo))
            .branch(dptree::case![State::ConfirmProfile { draft }].endpoint(confirm_profile))
            .branch(dptree::case![State::EditMenu { profile }].endpoint(edit_profile_menu))
            .branch(dptree::case![State::SettingsMenu { profile }].endpoint(settings_menu))
            .branch(dptree::case![State::WaitingForPhotoEdit { draft }].endpoint(edit_photo))
            .branch(
                dptree::case![State::WaitingForDescriptionEdit { draft }]
                    .endpoint(edit_description),
            )
            .branch(
                dptree::case![State::AwaitingIncomingLikeDecision {
                    profile,
                    liked_you_user_id
                }]
                .endpoint(handle_incoming_like_decision),
            )
            .branch(
                dptree::case![State::AwaitingProfileAction {
                    profile,
                    shown_profile_user_id,
                    return_to_menu
                }]
                .endpoint(handle_profile_action),
            )
            .branch(dptree::case![State::MainMenu { profile }].endpoint(main_menu)),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new(), pool, geocoder])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

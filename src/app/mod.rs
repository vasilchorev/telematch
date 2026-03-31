pub mod chat_ui;
pub mod handlers;
pub mod types;

pub use types::{AppResult, Language};

use crate::app::chat_ui::{is_incoming_like_decision_message, is_start_command};
use crate::app::handlers::{
    handle_age_input, handle_description_edit, handle_description_step, handle_edit_menu,
    handle_gender_selection, handle_global_incoming_like_decision, handle_incoming_like_decision,
    handle_initial_state, handle_language_preference_selection, handle_language_selection,
    handle_location_choice_selection, handle_location_input, handle_main_menu,
    handle_match_preference_selection, handle_name_input, handle_photo_edit, handle_photo_step,
    handle_profile_action, handle_profile_confirmation, handle_settings_menu, handle_start_command,
};
use crate::app::types::State;
use crate::db::connect_db;
use sqlx::PgPool;
use teloxide::{dispatching::dialogue::InMemStorage, dptree, prelude::*};

pub async fn run() -> AppResult<()> {
    dotenvy::dotenv().ok();

    pretty_env_logger::init();
    log::info!("Starting TeleMatch bot");

    let bot = Bot::from_env();
    let database_url = std::env::var("DATABASE_URL")?;
    let geocoder = crate::services::Geocoder::new()?;
    let pool: PgPool = connect_db(&database_url).await?;

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::filter(is_start_command).endpoint(handle_start_command))
            .branch(
                dptree::filter(is_incoming_like_decision_message)
                    .endpoint(handle_global_incoming_like_decision),
            )
            .branch(dptree::case![State::Start].endpoint(handle_initial_state))
            .branch(dptree::case![State::WaitingForLanguage].endpoint(handle_language_selection))
            .branch(
                dptree::case![State::WaitingForLanguagePreference { profile }]
                    .endpoint(handle_language_preference_selection),
            )
            .branch(dptree::case![State::WaitingForName { draft }].endpoint(handle_name_input))
            .branch(
                dptree::case![State::WaitingForGender { draft }].endpoint(handle_gender_selection),
            )
            .branch(
                dptree::case![State::WaitingForLookingFor { draft }]
                    .endpoint(handle_match_preference_selection),
            )
            .branch(dptree::case![State::WaitingForAge { draft }].endpoint(handle_age_input))
            .branch(
                dptree::case![State::WaitingForLocation { draft }].endpoint(handle_location_input),
            )
            .branch(
                dptree::case![State::WaitingForLocationChoice { draft, candidates }]
                    .endpoint(handle_location_choice_selection),
            )
            .branch(
                dptree::case![State::WaitingForDescription { draft }]
                    .endpoint(handle_description_step),
            )
            .branch(dptree::case![State::WaitingForPhoto { draft }].endpoint(handle_photo_step))
            .branch(
                dptree::case![State::ConfirmProfile { draft }]
                    .endpoint(handle_profile_confirmation),
            )
            .branch(dptree::case![State::EditMenu { profile }].endpoint(handle_edit_menu))
            .branch(dptree::case![State::SettingsMenu { profile }].endpoint(handle_settings_menu))
            .branch(dptree::case![State::WaitingForPhotoEdit { draft }].endpoint(handle_photo_edit))
            .branch(
                dptree::case![State::WaitingForDescriptionEdit { draft }]
                    .endpoint(handle_description_edit),
            )
            .branch(
                dptree::case![State::AwaitingIncomingLikeDecision {
                    profile,
                    incoming_like_user_id,
                    pending_like_count,
                    target_kind
                }]
                .endpoint(handle_incoming_like_decision),
            )
            .branch(
                dptree::case![State::AwaitingProfileAction {
                    profile,
                    displayed_profile_user_id,
                    return_to_main_menu
                }]
                .endpoint(handle_profile_action),
            )
            .branch(dptree::case![State::MainMenu { profile }].endpoint(handle_main_menu)),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new(), pool, geocoder])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    Ok(())
}

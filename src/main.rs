use enterprise_bot_core::adapters::telegram::BotState;
use enterprise_bot_core::adapters::telegram::router::start_bot;
use enterprise_bot_core::application::usecases::commands::register_user::RegisterUserCommand;
use enterprise_bot_core::application::usecases::commands::start_trial::StartTrialCommand;
use enterprise_bot_core::application::usecases::queries::get_menu_state::GetMenuStateQuery;
use enterprise_bot_core::application::usecases::queries::get_user::GetUserQuery;
use enterprise_bot_core::domain::user::value_objects::Money;
use enterprise_bot_core::infrastructure::config::AppConfig;
use enterprise_bot_core::infrastructure::database::*;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;
use tracing::info;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Loading configuration...");

    let config: AppConfig = AppConfig::load()?;
    let pool: PgPool = create_pg_pool(&config.general.database_url).await?;

    let user_repository = Arc::new(SqlxUserRepository::new(pool.clone()));
    let subscription_repository = Arc::new(SqlxSubscriptionRepository::new(pool.clone()));
    let uow = Arc::new(SqlxUnitOfWork::new(pool.clone()));

    let register_user_cmd = Arc::new(RegisterUserCommand::new(
        user_repository.clone(),
        config.vpn.uuid_namespace,
        Money(config.payments.base_price),
    ));
    let start_trial_cmd = Arc::new(StartTrialCommand::new(uow));
    let get_user_query = Arc::new(GetUserQuery::new(user_repository.clone()));
    let get_menu_state_query = Arc::new(GetMenuStateQuery::new(subscription_repository.clone()));

    info!("Connecting to Telegram...");
    let bot = Bot::new(&config.general.telegram_token);

    let me = bot
        .get_me()
        .await
        .map_err(|e| anyhow::anyhow!("Some message: {}", e))?;
    let bot_username = me.username().to_string();
    info!(bot_username, "Some message");

    let bot_state = BotState {
        register_user_cmd,
        start_trial_cmd,
        get_user_query,
        get_menu_state_query,
        bot_username,
        user_states: Arc::new(Mutex::new(HashMap::new())),
        broadcasting_admins: Arc::new(Mutex::new(HashSet::new())),
        admin_chat_id: config.general.admin_chat_id,
    };

    start_bot(bot, bot_state).await;

    Ok(())
}

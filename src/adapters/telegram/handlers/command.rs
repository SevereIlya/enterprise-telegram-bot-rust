use crate::adapters::telegram::{views, BotState};
use crate::adapters::telegram::commands::*;
use crate::adapters::telegram::error::TelegramResult;
use crate::domain::user::entity::User;
use teloxide::prelude::*;
use teloxide::types::ParseMode::Html;
use tracing::error;

pub async fn handle_commands(
    bot: Bot,
    msg: Message,
    state: BotState,
    user_opt: Option<User>,
    cmd: Command,
) -> TelegramResult<()> {
    let chat_id = msg.chat.id;
    let telegram_id = chat_id.0;

    match cmd {
        Command::Start(_payload) => {
            let username: String = msg.chat.username().unwrap_or("").to_string();
            let full_name: String = [msg.chat.first_name(), msg.chat.last_name()]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ");

            match state.register_user_cmd.execute(telegram_id, username, full_name).await {
                Ok(user) => {
                    let menu_state = state.get_menu_state_query.execute(&user).await?;
                    let view = views::build_start_message(menu_state.can_trial);
                    bot.send_message(chat_id, view.text)
                        .parse_mode(Html)
                        .reply_markup(view.keyboard)
                        .await?;
                }
                Err(e) => {
                    error!(error = ?e, "Ошибка при обработке /start");
                    bot.send_message(chat_id, e.message_error()).await?;
                }
            }
        }
        Command::Admin => {
            if let Some(user) = user_opt {
                if user.is_admin() {
                    // TODO
                } else {
                    bot.send_message(chat_id, "Команда не найдена.").await?;
                }
            } else {
                bot.send_message(chat_id, "❌ Сначала напиши /start").await?;
            }
        }
    }
    Ok(())
}

use super::command::handle_commands;
use super::state::{try_handle_broadcast, try_handle_user_state};
use crate::adapters::telegram::BotState;
use crate::adapters::telegram::commands::Command;
use crate::adapters::telegram::error::TelegramResult;
use teloxide::Bot;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

pub async fn message_handler(bot: Bot, msg: Message, state: BotState) -> TelegramResult<()> {
    let chat_id = msg.chat.id;
    let telegram_id = chat_id.0;

    let user_opt = state.get_user_query.execute(telegram_id).await?;
    
    if try_handle_broadcast(&bot, &msg, &state, &user_opt).await? {
        return Ok(());
    }
    
    if try_handle_user_state(&bot, &msg, &state, &user_opt).await? {
        return Ok(());
    }

    if let Some(text) = msg.text() {
        if let Ok(cmd) = Command::parse(text, &state.bot_username) {
            // Обрабатываем конкретную команду
            handle_commands(bot, msg, state, user_opt, cmd).await?;
            return Ok(());
        }
    }

    bot.send_message(
        chat_id,
        "🤷‍♂️ Я понимаю только команды и меню. Отправь /start",
    )
    .await?;

    Ok(())
}

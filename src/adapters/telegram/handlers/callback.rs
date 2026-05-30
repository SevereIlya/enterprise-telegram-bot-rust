use super::*;
use crate::adapters::telegram::BotState;
use crate::adapters::telegram::callbacks::*;
use crate::adapters::telegram::error::TelegramResult;
use teloxide::prelude::*;
use tracing::warn;

pub async fn callback_handler(bot: Bot, qry: CallbackQuery, state: BotState) -> TelegramResult<()> {
    let data = match qry.data.clone() {
        Some(data) => data,
        None => return Ok(()),
    };

    let msg = match qry.message.clone() {
        Some(msg) => msg,
        None => return Ok(()),
    };

    let action = CallbackAction::parse(&data);

    match action {
        CallbackAction::Menu(menu_action) => {
            menu::handle_menu(bot.clone(), &qry, msg, state, menu_action).await?;
        }
        CallbackAction::Ignore => {}
        CallbackAction::Unknown(unparsed) => {
            warn!(
                callback = unparsed,
                "An unknown or broken button arrived"
            );
        }
    }

    bot.answer_callback_query(qry.id).await?;
    Ok(())
}

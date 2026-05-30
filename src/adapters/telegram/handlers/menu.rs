use super::*;
use crate::adapters::telegram::BotState;
use crate::adapters::telegram::callbacks::MenuAction;
use crate::adapters::telegram::error::TelegramResult;
use crate::domain::error::DomainError;
use teloxide::prelude::*;
use teloxide::types::{MaybeInaccessibleMessage, ParseMode::Html};
use tracing::warn;

pub async fn handle_menu(
    bot: Bot,
    _qry: &CallbackQuery,
    msg: MaybeInaccessibleMessage,
    state: BotState,
    action: MenuAction,
) -> TelegramResult<()> {
    match action {
        MenuAction::StartTrial => handle_start_trial(bot, msg, state).await?,
        MenuAction::Router => {}
        MenuAction::Profile => {}
        MenuAction::Tariffs => {}
        MenuAction::Referral => {}
        MenuAction::Help => {}
        MenuAction::Down => show_main_menu(bot, msg, state, true).await?,
        MenuAction::Main => show_main_menu(bot, msg, state, false).await?,
    }
    Ok(())
}

// ============================================================================================== //

pub async fn handle_start_trial(
    bot: Bot,
    msg: MaybeInaccessibleMessage,
    state: BotState,
) -> TelegramResult<()> {
    let chat_id = msg.chat().id;
    let message_id = msg.id();
    let telegram_id = chat_id.0;

    let user = match state.get_user_query.execute(telegram_id).await? {
        Some(user) => user,
        None => {
            warn!(error = ?DomainError::UserNotFound, telegram_id, "Пользователь не найден");
            bot.send_message(chat_id, DomainError::UserNotFound.message_error()).await?;
            return Ok(());
        }
    };

    match state.start_trial_cmd.execute(user).await {
        Ok(subscription) => {
            let view = views::build_trial_success_view(subscription.expires_at);

            let is_media = msg
                .regular_message()
                .map(|m| m.photo().is_some() || m.document().is_some())
                .unwrap_or(false);

            if is_media {
                let _ = bot.delete_message(chat_id, message_id).await;
                bot.send_message(chat_id, view.text)
                    .parse_mode(Html)
                    .reply_markup(view.keyboard)
                    .await?;
            } else {
                bot.edit_message_text(chat_id, message_id, view.text)
                    .parse_mode(Html)
                    .reply_markup(view.keyboard)
                    .await?;
            }
        }
        Err(e) => {
            warn!(error = ?e, telegram_id, "Отказ в выдаче триала");
            bot.send_message(chat_id, e.message_error())
                .parse_mode(Html)
                .await?;
        }
    }
    Ok(())
}

pub async fn show_main_menu(
    bot: Bot,
    msg: MaybeInaccessibleMessage,
    state: BotState,
    drop_down: bool,
) -> TelegramResult<()> {
    let chat_id = msg.chat().id;
    let message_id = msg.id();
    let telegram_id = chat_id.0;

    let user = match state.get_user_query.execute(telegram_id).await? {
        Some(user) => user,
        None => {
            warn!(error = ?DomainError::UserNotFound, telegram_id, "Пользователь не найден");
            bot.send_message(chat_id, DomainError::UserNotFound.message_error()).await?;
            return Ok(());
        }
    };

    let menu_state = state.get_menu_state_query.execute(&user).await?;

    let view = if drop_down {
        views::build_refresh_menu_view(menu_state.can_trial)
    } else {
        views::build_main_menu_view(menu_state.can_trial)
    };

    let is_media = msg
        .regular_message()
        .map(|m| m.photo().is_some() || m.document().is_some())
        .unwrap_or(false);

    if drop_down || is_media {
        let _ = bot.delete_message(chat_id, message_id).await?;
        bot.send_message(chat_id, view.text)
            .parse_mode(Html)
            .reply_markup(view.keyboard)
            .await?;
    } else {
        bot.edit_message_text(chat_id, message_id, view.text)
            .parse_mode(Html)
            .reply_markup(view.keyboard)
            .await?;
    }
    Ok(())
}

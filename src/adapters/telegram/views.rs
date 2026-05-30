use crate::adapters::telegram::handlers::MessageView;
use chrono::{DateTime, FixedOffset, Utc};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn build_start_message(can_trial: bool) -> MessageView {
    let text = "Welcome to the Service. Please choose an option below.".to_string();

    let mut rows = Vec::new();

    if can_trial {
        rows.push(vec![InlineKeyboardButton::callback(
            "🎁 Try it free for 5 days!",
            "menu:trial",
        )])
    } else {
        rows.push(vec![InlineKeyboardButton::callback(
            "🚀 Select an action",
            "menu:router",
        )])
    };

    rows.push(vec![InlineKeyboardButton::callback(
        "🏠 Main Menu",
        "menu:main",
    )]);

    MessageView {
        text,
        keyboard: InlineKeyboardMarkup::new(rows),
    }
}

pub fn build_trial_success_view(expires_at: DateTime<Utc>) -> MessageView {
    let msk_offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let expires_at_msk = expires_at.with_timezone(&msk_offset);
    let date_str = expires_at_msk.format("%d.%m.%Y %H:%M").to_string();

    let text = format!("Your 5-day trial has been activated. Enjoy the service until {}", date_str).trim().to_string();

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "🚀 Select an action",
            "menu:router",
        )],
        vec![InlineKeyboardButton::callback(
            "🏠 Main Menu",
            "menu:main",
        )],
    ]);

    MessageView { text, keyboard }
}

pub fn build_main_menu_view(can_trial: bool) -> MessageView {
    let text = "🏠 <b>Main Menu</b>\n\nSelect the desired action:".to_string();
    MessageView {
        text,
        keyboard: build_menu_keyboard(can_trial),
    }
}

pub fn build_refresh_menu_view(can_trial: bool) -> MessageView {
    let text =
        "<i>The menu has been updated 👇</i>\n\n🏠 <b>Main Menu</b>\n\nSelect the desired action:".to_string();
    MessageView {
        text,
        keyboard: build_menu_keyboard(can_trial),
    }
}

// ============================================================================================== //

fn build_menu_keyboard(can_trial: bool) -> InlineKeyboardMarkup {
    let mut rows = Vec::new();

    rows.push(vec![InlineKeyboardButton::callback(
        "🚀 Select an action",
        "menu:router",
    )]);
    rows.push(vec![
        InlineKeyboardButton::callback("👤 Профиль", "menu:profile"),
        InlineKeyboardButton::callback("🛒 Тарифы", "menu:tariffs"),
    ]);
    rows.push(vec![InlineKeyboardButton::callback(
        "🎁 Invite a friend",
        "menu:referral",
    )]);
    rows.push(vec![InlineKeyboardButton::callback(
        "🛠 Help",
        "menu:help",
    )]);
    rows.push(vec![InlineKeyboardButton::callback(
        "🔄 Update / Down",
        "menu:down",
    )]);

    if can_trial {
        rows.insert(
            0,
            vec![InlineKeyboardButton::callback(
                "🎁 Try it for free (5 days)",
                "menu:trial",
            )],
        )
    }

    InlineKeyboardMarkup::new(rows)
}

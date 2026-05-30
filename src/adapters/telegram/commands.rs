use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Launch the bot and open the menu")]
    Start(String),

    #[command(hide, description = "Administrator Panel")]
    Admin,
}
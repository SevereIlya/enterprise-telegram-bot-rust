use crate::application::error::AppResult;
use crate::domain::user::entity::User;
use crate::domain::user::repository::DynUserRepository;
use crate::domain::user::value_objects::TelegramId;

pub struct GetUserQuery {
    user_repo: DynUserRepository,
}

impl GetUserQuery {
    pub fn new(user_repo: DynUserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, telegram_id: i64) -> AppResult<Option<User>> {
        let user = self.user_repo.find_by_telegram_id(TelegramId(telegram_id)).await?;
        Ok(user)
    }
}
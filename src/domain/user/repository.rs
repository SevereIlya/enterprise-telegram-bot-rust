use std::sync::Arc;
use super::entity::User;
use super::value_objects::{TelegramId, UserId};
use crate::domain::error::DomainResult;
use async_trait::async_trait;

pub type DynUserRepository = Arc<dyn UserRepository + Send + Sync>;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> DomainResult<UserId>;
    async fn update(&self, user: &User) -> DomainResult<()>;
    async fn find_by_user_id(&self, user_id: UserId) -> DomainResult<Option<User>>;
    async fn find_by_telegram_id(&self, telegram_id: TelegramId) -> DomainResult<Option<User>>;
}

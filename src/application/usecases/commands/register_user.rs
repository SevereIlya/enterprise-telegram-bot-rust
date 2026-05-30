use crate::application::error::*;
use crate::domain::error::*;
use crate::domain::user::entity::*;
use crate::domain::user::repository::*;
use crate::domain::user::value_objects::*;
use tracing::{info, warn};
use uuid::Uuid;

pub struct RegisterUserCommand {
    user_repo: DynUserRepository,
    uuid_namespace: Uuid,
    base_price: Money,
}

impl RegisterUserCommand {
    pub fn new(user_repo: DynUserRepository, uuid_namespace: Uuid, base_price: Money) -> Self {
        Self {
            user_repo,
            uuid_namespace,
            base_price,
        }
    }

    pub async fn execute(
        &self,
        telegram_id: i64,
        username: String,
        full_name: String,
    ) -> AppResult<User> {
        let telegram_id = TelegramId(telegram_id);

        if let Some(user) = self.user_repo.find_by_telegram_id(telegram_id).await? {
            return Ok(user);
        }

        let user_uuid = Uuid::new_v5(&self.uuid_namespace, telegram_id.to_string().as_bytes());

        const MAX_RETRIES: u8 = 5;
        let mut attempts = 0;

        loop {
            attempts += 1;

            let ref_code = ReferralCode::generate();
            let sub_token = SubscriptionToken::generate();

            let new_user = User::new(
                telegram_id,
                user_uuid,
                username.clone(),
                full_name.clone(),
                self.base_price,
                ref_code,
                sub_token,
            );

            match self.user_repo.create(&new_user).await {
                Ok(inserted_id) => {
                    info!(
                        telegram_id = %telegram_id,
                        username = %username,
                        "Some message"
                    );

                    let mut saved_user = new_user;
                    saved_user.user_id = Some(inserted_id);

                    return Ok(saved_user);
                }
                Err(DomainError::ReferralCodeCollision) => {
                    if attempts >= MAX_RETRIES {
                        return Err(AppError::MaxRetriesExceeded(
                            "Some message".into(),
                        ));
                    }
                    warn!(attempts, "Some message");
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    
    struct MockUserRepository {
        existing_user: Option<User>,
        collisions_to_simulate: Mutex<u8>,
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn create(&self, _user: &User) -> DomainResult<UserId> {
            let mut cols = self.collisions_to_simulate.lock().unwrap();
            if *cols > 0 {
                *cols -= 1;
                return Err(DomainError::ReferralCodeCollision);
            }
            Ok(UserId(99))
        }
        async fn update(&self, _user: &User) -> DomainResult<()> {
            Ok(())
        }
        async fn find_by_user_id(&self, _user_id: UserId) -> DomainResult<Option<User>> {
            Ok(None)
        }
        async fn find_by_telegram_id(
            &self,
            _telegram_id: TelegramId,
        ) -> DomainResult<Option<User>> {
            Ok(self.existing_user.clone())
        }
    }

    fn setup_command(existing_user: Option<User>, collisions: u8) -> RegisterUserCommand {
        let mock_repo = Arc::new(MockUserRepository {
            existing_user,
            collisions_to_simulate: Mutex::new(collisions),
        });
        RegisterUserCommand::new(mock_repo, uuid::Uuid::new_v4(), Money(15000))
    }

    #[tokio::test]
    async fn test_register_new_user_success() {
        // === ARRANGE
        let cmd = setup_command(None, 0);
        // === ACT
        let result = cmd
            .execute(123, "test_user".into(), "Test User".into())
            .await;
        // === ASSERT
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_id, Some(UserId(99)));
        assert_eq!(user.frozen_base_price, Money(15000));
    }

    #[tokio::test]
    async fn test_returns_existing_user() {
        // === ARRANGE
        let mut existing_user = User::new(
            TelegramId(123),
            Uuid::new_v4(),
            "old".into(),
            "Old".into(),
            Money(10),
            ReferralCode("A".into()),
            SubscriptionToken("B".into()),
        );
        existing_user.user_id = Some(UserId(42));
        let cmd = setup_command(Some(existing_user), 0);
        // === ACT
        let result = cmd
            .execute(123, "test_user".into(), "Test User".into())
            .await;
        // === ASSERT
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().user_id,
            Some(UserId(42))
        );
    }

    #[tokio::test]
    async fn test_retry_works_on_referral_collision() {
        // === ARRANGE
        let cmd = setup_command(None, 2);
        // === ACT
        let result = cmd
            .execute(123, "test_user".into(), "Test User".into())
            .await;
        // === ASSERT
        assert!(
            result.is_ok()
        );
        assert_eq!(result.unwrap().user_id, Some(UserId(99)));
    }

    #[tokio::test]
    async fn test_fails_after_max_retries() {
        // === ARRANGE
        let cmd = setup_command(None, 10);
        // === ACT
        let result = cmd
            .execute(123, "test_user".into(), "Test User".into())
            .await;
        // === ASSERT
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::MaxRetriesExceeded(_)
        ));
    }
}

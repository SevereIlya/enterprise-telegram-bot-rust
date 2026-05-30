use crate::application::error::*;
use crate::domain::error::*;
use crate::domain::subscription::entity::*;
use crate::domain::subscription::value_objects::*;
use crate::domain::uow::repository::*;
use crate::domain::user::entity::*;
use chrono::{Days, Utc};

pub struct StartTrialCommand {
    uow: DynUnitOfWork,
}

impl StartTrialCommand {
    pub fn new(uow: DynUnitOfWork) -> Self {
        Self { uow }
    }

    pub async fn execute(&self, mut user: User) -> AppResult<Subscription> {
        let user_id = user.user_id.ok_or(DomainError::EntityNotSaved)?;

        let mut tx: BoxedUowContext = self.uow.begin().await?;

        if tx
            .subscriptions()
            .find_active_by_user_id(user_id)
            .await?
            .is_some()
        {
            tx.rollback().await?;
            return Err(DomainError::AlreadyHasSubscription.into());
        }

        user.use_trial()?;

        let sub = Subscription::new(
            user_id,
            SubscriptionPlan::Trial,
            Utc::now(),
            Utc::now() + Days::new(5),
            SubscriptionStatus::Active,
            SubscriptionDevices(2),
        );

        tx.subscriptions().create(&sub).await?;
        tx.users().update(&user).await?;

        tx.commit().await?;

        Ok(sub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::subscription::repository::*;
    use crate::domain::user::repository::*;
    use crate::domain::user::value_objects::*;
    use async_trait::async_trait;
    use chrono::Months;
    use std::sync::Arc;
    use uuid::Uuid;
    
    struct MockUserRepository;

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn create(&self, _user: &User) -> DomainResult<UserId> {
            Ok(UserId(1))
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
            Ok(None)
        }
    }

    struct MockSubscriptionRepository {
        has_active_sub: bool,
    }

    #[async_trait]
    impl SubscriptionRepository for MockSubscriptionRepository {
        async fn create(&self, _subscription: &Subscription) -> DomainResult<()> {
            Ok(())
        }
        async fn find_active_by_user_id(
            &self,
            _user_id: UserId,
        ) -> DomainResult<Option<Subscription>> {
            if self.has_active_sub {
                let sub = Subscription::new(
                    UserId(12),
                    SubscriptionPlan::Month3,
                    Utc::now(),
                    Utc::now() + Months::new(3),
                    SubscriptionStatus::Active,
                    SubscriptionDevices(2),
                );
                Ok(Some(sub))
            } else {
                Ok(None)
            }
        }
    }
    
    struct MockUowContext {
        has_active_sub: bool,
    }

    #[async_trait]
    impl UowContext for MockUowContext {
        fn users(&self) -> DynUserRepository {
            Arc::new(MockUserRepository)
        }
        fn subscriptions(&self) -> DynSubscriptionRepository {
            Arc::new(MockSubscriptionRepository {
                has_active_sub: self.has_active_sub,
            })
        }
        async fn commit(&mut self) -> DomainResult<()> {
            Ok(())
        }
        async fn rollback(&mut self) -> DomainResult<()> {
            Ok(())
        }
    }

    struct MockUnitOfWork {
        has_active_sub: bool,
    }

    #[async_trait]
    impl UnitOfWork for MockUnitOfWork {
        async fn begin(&self) -> DomainResult<BoxedUowContext> {
            Ok(Box::new(MockUowContext {
                has_active_sub: self.has_active_sub,
            }))
        }
    }
    
    fn create_test_user(has_id: bool, trial_used: bool) -> User {
        let mut user = User::new(
            TelegramId(123),
            Uuid::new_v4(),
            "test".into(),
            "Test User".into(),
            Money(15000),
            ReferralCode("REF".into()),
            SubscriptionToken("TOK".into()),
        );
        if has_id {
            user.user_id = Some(UserId(1));
        }
        user.trial_used = trial_used;
        user
    }

    fn setup_command(has_active_sub: bool) -> StartTrialCommand {
        let mock_uow = Arc::new(MockUnitOfWork { has_active_sub });
        StartTrialCommand::new(mock_uow)
    }
    
    #[tokio::test]
    async fn test_start_trial_success() {
        // === ARRANGE
        let cmd = setup_command(false);
        let user = create_test_user(true, false);
        // === ACT
        let result = cmd.execute(user).await;
        // === ASSERT
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fails_if_user_not_saved_in_db() {
        // === ARRANGE
        let cmd = setup_command(false);
        let user = create_test_user(false, false);
        // === ACT
        let result = cmd.execute(user).await;
        // === ASSERT
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::Domain(DomainError::EntityNotSaved)
        ));
    }

    #[tokio::test]
    async fn test_fails_if_trial_already_used() {
        // === ARRANGE
        let cmd = setup_command(false);
        let user = create_test_user(true, true);
        // === ACT
        let result = cmd.execute(user).await;
        // === ASSERT
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::Domain(DomainError::TrialAlreadyUsed)
        ));
    }

    #[tokio::test]
    async fn test_fails_if_user_has_active_subscription() {
        // === ARRANGE
        let cmd = setup_command(true);
        let user = create_test_user(true, false);
        // === ACT
        let result = cmd.execute(user).await;
        // === ASSERT
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::Domain(DomainError::AlreadyHasSubscription)
        ));
    }
}

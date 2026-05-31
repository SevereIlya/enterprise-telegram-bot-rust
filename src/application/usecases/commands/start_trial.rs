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
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    struct MockUserRepository {
        updated_users: Arc<Mutex<Vec<User>>>,
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn create(&self, _user: &User) -> DomainResult<UserId> {
            Ok(UserId(1))
        }
        async fn update(&self, user: &User) -> DomainResult<()> {
            self.updated_users.lock().unwrap().push(user.clone());
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
        created_subscriptions: Arc<Mutex<Vec<Subscription>>>,
    }

    #[async_trait]
    impl SubscriptionRepository for MockSubscriptionRepository {
        async fn create(&self, subscription: &Subscription) -> DomainResult<()> {
            self.created_subscriptions
                .lock()
                .unwrap()
                .push(subscription.clone());
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
        user_repo: Arc<MockUserRepository>,
        sub_repo: Arc<MockSubscriptionRepository>,
        committed: Arc<Mutex<bool>>,
        rolled_back: Arc<Mutex<bool>>,
    }

    #[async_trait]
    impl UowContext for MockUowContext {
        fn users(&self) -> DynUserRepository {
            self.user_repo.clone()
        }
        fn subscriptions(&self) -> DynSubscriptionRepository {
            self.sub_repo.clone()
        }
        async fn commit(&mut self) -> DomainResult<()> {
            *self.committed.lock().unwrap() = true;
            Ok(())
        }
        async fn rollback(&mut self) -> DomainResult<()> {
            *self.rolled_back.lock().unwrap() = true;
            Ok(())
        }
    }

    struct MockUnitOfWork {
        user_repo: Arc<MockUserRepository>,
        sub_repo: Arc<MockSubscriptionRepository>,
        committed: Arc<Mutex<bool>>,
        rolled_back: Arc<Mutex<bool>>,
    }

    #[async_trait]
    impl UnitOfWork for MockUnitOfWork {
        async fn begin(&self) -> DomainResult<BoxedUowContext> {
            Ok(Box::new(MockUowContext {
                user_repo: self.user_repo.clone(),
                sub_repo: self.sub_repo.clone(),
                committed: self.committed.clone(),
                rolled_back: self.rolled_back.clone(),
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

    fn setup_command(
        has_active_sub: bool,
    ) -> (
        StartTrialCommand,
        Arc<MockUserRepository>,
        Arc<MockSubscriptionRepository>,
        Arc<Mutex<bool>>,
        Arc<Mutex<bool>>,
    ) {
        let user_repo = Arc::new(MockUserRepository {
            updated_users: Arc::new(Mutex::new(Vec::new())),
        });
        let sub_repo = Arc::new(MockSubscriptionRepository {
            has_active_sub,
            created_subscriptions: Arc::new(Mutex::new(Vec::new())),
        });
        let committed = Arc::new(Mutex::new(false));
        let rolled_back = Arc::new(Mutex::new(false));

        let mock_uow = Arc::new(MockUnitOfWork {
            user_repo: user_repo.clone(),
            sub_repo: sub_repo.clone(),
            committed: committed.clone(),
            rolled_back: rolled_back.clone(),
        });

        (
            StartTrialCommand::new(mock_uow),
            user_repo,
            sub_repo,
            committed,
            rolled_back,
        )
    }

    #[tokio::test]
    async fn test_start_trial_success() {
        // === ARRANGE
        let (cmd, user_repo, sub_repo, committed, rolled_back) = setup_command(false);
        let user = create_test_user(true, false);
        // === ACT
        let result = cmd.execute(user).await;
        // === ASSERT
        assert!(result.is_ok(), "Ожидали успешную выдачу триала");

        let subscription = result.unwrap();
        assert_eq!(
            subscription.plan,
            SubscriptionPlan::Trial,
            "План должен быть Trial"
        );
        assert_eq!(
            subscription.devices,
            SubscriptionDevices(2),
            "Должно быть 2 устройства"
        );
        assert_eq!(
            subscription.status,
            SubscriptionStatus::Active,
            "Статус должен быть Active"
        );

        // Проверяем, что пользователь был обновлён
        let updated_users = user_repo.updated_users.lock().unwrap();
        assert_eq!(
            updated_users.len(),
            1,
            "Пользователь должен быть обновлён один раз"
        );
        assert!(
            updated_users[0].trial_used,
            "Флаг trial_used должен быть установлен"
        );

        // Проверяем, что подписка была создана
        let created_subs = sub_repo.created_subscriptions.lock().unwrap();
        assert_eq!(
            created_subs.len(),
            1,
            "Подписка должна быть создана один раз"
        );

        // Проверяем, что транзакция была закоммичена
        assert!(
            *committed.lock().unwrap(),
            "Транзакция должна быть закоммичена"
        );
        assert!(
            !*rolled_back.lock().unwrap(),
            "Транзакция не должна быть откачена"
        );
    }

    #[tokio::test]
    async fn test_fails_if_user_not_saved_in_db() {
        // === ARRANGE
        let (cmd, _user_repo, _sub_repo, committed, _rolled_back) = setup_command(false);
        let user = create_test_user(false, false);
        // === ACT
        let result = cmd.execute(user).await;
        // === ASSERT
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::Domain(DomainError::EntityNotSaved)
        ));

        // Проверяем, что транзакция не была закоммичена
        assert!(
            !*committed.lock().unwrap(),
            "Транзакция не должна быть закоммичена при ошибке"
        );
    }

    #[tokio::test]
    async fn test_fails_if_trial_already_used() {
        // === ARRANGE
        let (cmd, _user_repo, _sub_repo, committed, _rolled_back) = setup_command(false);
        let user = create_test_user(true, true);
        // === ACT
        let result = cmd.execute(user).await;
        // === ASSERT
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::Domain(DomainError::TrialAlreadyUsed)
        ));

        // Проверяем, что транзакция не была закоммичена
        assert!(
            !*committed.lock().unwrap(),
            "Транзакция не должна быть закоммичена при ошибке"
        );
    }

    #[tokio::test]
    async fn test_fails_if_user_has_active_subscription() {
        // === ARRANGE
        let (cmd, _user_repo, _sub_repo, committed, rolled_back) = setup_command(true);
        let user = create_test_user(true, false);
        // === ACT
        let result = cmd.execute(user).await;
        // === ASSERT
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::Domain(DomainError::AlreadyHasSubscription)
        ));

        // rollback должен быть вызван
        assert!(
            *rolled_back.lock().unwrap(),
            "rollback() должен быть вызван при наличии активной подписки"
        );
        assert!(
            !*committed.lock().unwrap(),
            "Транзакция не должна быть закоммичена при ошибке"
        );
    }
}

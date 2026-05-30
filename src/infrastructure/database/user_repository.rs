use super::*;
use crate::domain::error::*;
use crate::domain::user::entity::*;
use crate::domain::user::repository::*;
use crate::domain::user::value_objects::*;
use async_trait::async_trait;
use tracing::instrument;

pub struct SqlxUserRepository {
    executor: SqlxExecutor,
}

impl SqlxUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            executor: SqlxExecutor::Pool(pool),
        }
    }

    pub fn transaction(tx: SharedTransaction) -> Self {
        Self {
            executor: SqlxExecutor::Transaction(tx),
        }
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    #[instrument(skip(self, user), fields(telegram_id = %user.telegram_id))]
    async fn create(&self, user: &User) -> DomainResult<UserId> {
        let query = sqlx::query!(
            r#"
            INSERT INTO users (
                telegram_id, uuid, username, full_name, role,
                frozen_base_price, referral_code, subscription_token,
                trial_used, first_purchase_discount, personal_discount, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING user_id
            "#,
            user.telegram_id.0,
            user.uuid,
            user.username,
            user.full_name,
            user.role.as_str(),
            user.frozen_base_price.0,
            user.referral_code.0,
            user.subscription_token.0,
            user.trial_used,
            user.first_purchase_discount.0 as i32,
            user.personal_discount.0 as i32,
            user.created_at,
        );

        let result = match &self.executor {
            SqlxExecutor::Pool(pool) => query.fetch_one(pool).await,
            SqlxExecutor::Transaction(tx_mutex) => {
                let mut lock = tx_mutex.lock().await;
                if let Some(tx) = lock.as_mut() {
                    query.fetch_one(&mut **tx).await
                } else {
                    return Err(DomainError::Transaction("Some message".into()));
                }
            }
        };

        match result {
            Ok(record) => Ok(UserId(record.user_id)),
            Err(e) => {
                if let sqlx::Error::Database(db_err) = &e {
                    if db_err.code().as_deref() == Some("23505")
                        && db_err.constraint().as_deref() == Some("users_referral_code_unique")
                    {
                        return Err(DomainError::ReferralCodeCollision);
                    }
                }
                Err(DomainError::Repository(e.to_string()))
            }
        }
    }

    #[instrument(skip(self, user), fields(telegram_id = %user.telegram_id))]
    async fn update(&self, user: &User) -> DomainResult<()> {
        let user_id = user.user_id.ok_or(DomainError::EntityNotSaved)?;

        let query = sqlx::query!(
            r#"
            UPDATE users
            SET role = $1,
                frozen_base_price = $2,
                trial_used = $3,
                first_purchase_discount = $4,
                personal_discount = $5
            WHERE user_id = $6
            "#,
            user.role.as_str(),
            user.frozen_base_price.0,
            user.trial_used,
            user.first_purchase_discount.0 as i32,
            user.personal_discount.0 as i32,
            user_id.0,
        );

        let result = match &self.executor {
            SqlxExecutor::Pool(pool) => query.execute(pool).await,
            SqlxExecutor::Transaction(tx_mutex) => {
                let mut lock = tx_mutex.lock().await;
                if let Some(tx) = lock.as_mut() {
                    query.execute(&mut **tx).await
                } else {
                    return Err(DomainError::Transaction("Some message".into()));
                }
            }
        };
        result.map_err(|e| DomainError::Repository(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn find_by_user_id(&self, user_id: UserId) -> DomainResult<Option<User>> {
        let query = sqlx::query_as!(
            UserRow,
            r#"
            SELECT *
            FROM users
            WHERE user_id = $1
            "#,
            user_id.0
        );

        let result = match &self.executor {
            SqlxExecutor::Pool(pool) => query.fetch_optional(pool).await,
            SqlxExecutor::Transaction(tx_mutex) => {
                let mut lock = tx_mutex.lock().await;
                if let Some(tx) = lock.as_mut() {
                    query.fetch_optional(&mut **tx).await
                } else {
                    return Err(DomainError::Transaction("Some message".into()));
                }
            }
        };

        let row: Option<UserRow> = result.map_err(|e| DomainError::Repository(e.to_string()))?;
        let user: Option<User> = row.map(TryInto::try_into).transpose()?;

        Ok(user)
    }

    #[instrument(skip(self))]
    async fn find_by_telegram_id(&self, telegram_id: TelegramId) -> DomainResult<Option<User>> {
        let query = sqlx::query_as!(
            UserRow,
            r#"
            SELECT *
            FROM users
            WHERE telegram_id = $1
            "#,
            telegram_id.0
        );

        let result = match &self.executor {
            SqlxExecutor::Pool(pool) => query.fetch_optional(pool).await,
            SqlxExecutor::Transaction(tx_mutex) => {
                let mut lock = tx_mutex.lock().await;
                if let Some(tx) = lock.as_mut() {
                    query.fetch_optional(&mut **tx).await
                } else {
                    return Err(DomainError::Transaction("Some message".into()));
                }
            }
        };

        let row: Option<UserRow> = result.map_err(|e| DomainError::Repository(e.to_string()))?;
        let user: Option<User> = row.map(TryInto::try_into).transpose()?;

        Ok(user)
    }
}

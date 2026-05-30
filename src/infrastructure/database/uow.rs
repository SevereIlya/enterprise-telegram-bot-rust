use super::*;
use crate::domain::error::*;
use crate::domain::subscription::repository::*;
use crate::domain::uow::repository::*;
use crate::domain::user::repository::*;
use async_trait::async_trait;
use tokio::sync::MutexGuard;

pub struct SqlxUnitOfWork {
    pool: PgPool,
}

impl SqlxUnitOfWork {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWork for SqlxUnitOfWork {
    async fn begin(&self) -> DomainResult<BoxedUowContext> {
        let tx: Transaction<Postgres> = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Transaction(e.to_string()))?;

        let shared_tx = Arc::new(Mutex::new(Some(tx)));

        Ok(Box::new(SqlxUowContext { tx: shared_tx }))
    }
}

pub struct SqlxUowContext {
    tx: SharedTransaction,
}

#[async_trait]
impl UowContext for SqlxUowContext {
    fn users(&self) -> DynUserRepository {
        Arc::new(SqlxUserRepository::transaction(self.tx.clone()))
    }

    fn subscriptions(&self) -> DynSubscriptionRepository {
        Arc::new(SqlxSubscriptionRepository::transaction(self.tx.clone()))
    }

    async fn commit(&mut self) -> DomainResult<()> {
        let mut tx_lock: MutexGuard<Option<Transaction<'static, Postgres>>> = self.tx.lock().await;
        if let Some(tx) = tx_lock.take() {
            let tx: Transaction<'static, Postgres> = tx;
            tx.commit()
                .await
                .map_err(|e| DomainError::Transaction(e.to_string()))?;
        }
        Ok(())
    }

    async fn rollback(&mut self) -> DomainResult<()> {
        let mut tx_lock = self.tx.lock().await;
        if let Some(tx) = tx_lock.take() {
            tx.rollback()
                .await
                .map_err(|e| DomainError::Transaction(e.to_string()))?;
        }
        Ok(())
    }
}

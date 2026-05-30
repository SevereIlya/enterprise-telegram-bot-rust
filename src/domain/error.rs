use thiserror::Error;

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Repository error: {0}")]
    Repository(String),
    #[error("Transaction error: {0}")]
    Transaction(String),
    #[error("Entity has no ID. It must be saved to the database first.")]
    EntityNotSaved,

    #[error("User not found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Referral code collision")]
    ReferralCodeCollision,
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    #[error("Invalid discount: {0}")]
    InvalidDiscount(i32),
    #[error("Trial already used")]
    TrialAlreadyUsed,

    #[error("Invalid subscription plan: {0}")]
    InvalidPlan(String),
    #[error("Invalid subscription status: {0}")]
    InvalidStatus(String),
    #[error("Already has subscription")]
    AlreadyHasSubscription,
}

impl DomainError {
    pub fn message_error(&self) -> &'static str {
        match self {
            DomainError::UserNotFound => "Some message",
            DomainError::TrialAlreadyUsed => "Some message",
            DomainError::AlreadyHasSubscription => "Some message",
            _ => "Some message",
        }
    }
}
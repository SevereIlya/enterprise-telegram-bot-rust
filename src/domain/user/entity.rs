use super::value_objects::*;
use crate::domain::error::{DomainError, DomainResult};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: Option<UserId>,
    pub telegram_id: TelegramId,
    pub uuid: Uuid,
    pub username: String,
    pub full_name: String,
    pub role: UserRole,
    pub frozen_base_price: Money,
    pub referral_code: ReferralCode,
    pub subscription_token: SubscriptionToken,
    pub trial_used: bool,
    pub first_purchase_discount: DiscountPercent,
    pub personal_discount: DiscountPercent,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        telegram_id: TelegramId,
        uuid: Uuid,
        username: String,
        full_name: String,
        frozen_base_price: Money,
        referral_code: ReferralCode,
        subscription_token: SubscriptionToken,
    ) -> Self {
        Self {
            user_id: None,
            telegram_id,
            uuid,
            username,
            full_name,
            role: UserRole::User,
            frozen_base_price,
            referral_code,
            subscription_token,
            trial_used: false,
            first_purchase_discount: DiscountPercent::zero(),
            personal_discount: DiscountPercent::zero(),
            created_at: Utc::now(),
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    pub fn use_trial(&mut self) -> DomainResult<()> {
        if self.trial_used {
            return Err(DomainError::TrialAlreadyUsed);
        }
        self.trial_used = true;
        self.first_purchase_discount = DiscountPercent::new(15);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_base_user() -> User {
        User::new(
            TelegramId(123456789),
            Uuid::new_v4(),
            "durov".to_string(),
            "Pavel Durov".to_string(),
            Money(20000),
            ReferralCode("MY_REF_123".to_string()),
            SubscriptionToken("TOKEN_XYZ".to_string()),
        )
    }

    #[test]
    fn test_user_new_creates_valid_default_user() {
        let user = create_base_user();
        assert_eq!(user.user_id, None);
        assert_eq!(user.telegram_id, TelegramId(123456789));
        assert_eq!(user.username, "durov".to_string());
        assert_eq!(user.full_name, "Pavel Durov".to_string());
        assert_eq!(user.role, UserRole::User);
        assert_eq!(user.frozen_base_price, Money(20000));
        assert_eq!(user.referral_code, ReferralCode("MY_REF_123".to_string()));
        assert_eq!(user.subscription_token, SubscriptionToken("TOKEN_XYZ".to_string()));
        assert_eq!(user.trial_used, false);
        assert_eq!(user.first_purchase_discount, DiscountPercent::zero());
        assert_eq!(user.personal_discount, DiscountPercent::zero());
    }

    #[test]
    fn test_use_trial_success() {
        let mut user = create_base_user();

        // Убеждаемся, что до вызова скидки нет
        assert_eq!(user.first_purchase_discount, DiscountPercent::zero());

        let result = user.use_trial();

        assert!(result.is_ok());
        assert_eq!(user.trial_used, true);
        assert_eq!(user.first_purchase_discount, DiscountPercent::new(15));
    }

    #[test]
    fn test_use_trial_fails_if_already_used() {
        let mut user = create_base_user();

        user.trial_used = true;

        let result = user.use_trial();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::TrialAlreadyUsed));
    }

    #[test]
    fn test_user_is_admin() {
        let mut user = create_base_user();
        assert_eq!(user.is_admin(), false);

        user.role = UserRole::Admin;
        assert_eq!(user.is_admin(), true);
    }
}

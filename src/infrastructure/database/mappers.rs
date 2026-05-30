use super::models::{SubscriptionRow, UserRow};
use crate::domain::error::DomainError;
use crate::domain::subscription::entity::Subscription;
use crate::domain::subscription::value_objects::*;
use crate::domain::user::entity::User;
use crate::domain::user::value_objects::*;

impl TryFrom<UserRow> for User {
    type Error = DomainError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            user_id: Some(UserId(row.user_id)),
            telegram_id: TelegramId(row.telegram_id),
            uuid: row.uuid,
            username: row.username,
            full_name: row.full_name,
            role: row.role.parse()?,
            frozen_base_price: Money(row.frozen_base_price),
            referral_code: ReferralCode(row.referral_code),
            subscription_token: SubscriptionToken(row.subscription_token),
            trial_used: row.trial_used,
            first_purchase_discount: DiscountPercent::new(row.first_purchase_discount),
            personal_discount: DiscountPercent::new(row.personal_discount),
            created_at: row.created_at,
        })
    }
}

impl TryFrom<SubscriptionRow> for Subscription {
    type Error = DomainError;

    fn try_from(row: SubscriptionRow) -> Result<Self, Self::Error> {
        Ok(Subscription {
            subscription_id: Some(SubscriptionId(row.subscription_id)),
            user_id: UserId(row.user_id),
            plan: row.plan.parse()?,
            starts_at: row.starts_at,
            expires_at: row.expires_at,
            status: row.status.parse()?,
            devices: SubscriptionDevices(row.devices),
            created_at: row.created_at,
        })
    }
}

use super::value_objects::*;
use crate::domain::user::value_objects::UserId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Subscription {
    pub subscription_id: Option<SubscriptionId>,
    pub user_id: UserId,
    pub plan: SubscriptionPlan,
    pub starts_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: SubscriptionStatus,
    pub devices: SubscriptionDevices,
    pub created_at: DateTime<Utc>,
}

impl Subscription {
    pub fn new(
        user_id: UserId,
        plan: SubscriptionPlan,
        starts_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        status: SubscriptionStatus,
        devices: SubscriptionDevices,
    ) -> Self {
        Self {
            subscription_id: None,
            user_id,
            plan,
            starts_at,
            expires_at,
            status,
            devices,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Months;
    use super::*;

    #[test]
    fn test_subscription_new_creates_valid_default_subscription() {
        let user_id = UserId(42);
        let plan = SubscriptionPlan::Month3;
        let starts_at = Utc::now();
        let expires_at = starts_at + Months::new(3);
        let status = SubscriptionStatus::Active;
        let devices = SubscriptionDevices(2);

        let subscription = Subscription::new(
            user_id,
            plan.clone(),
            starts_at,
            expires_at,
            status.clone(),
            devices
        );

        assert_eq!(subscription.subscription_id, None);
        assert_eq!(subscription.user_id, user_id);
        assert_eq!(subscription.plan, plan);
        assert_eq!(subscription.starts_at, starts_at);
        assert_eq!(subscription.expires_at, expires_at);
        assert_eq!(subscription.status, status);
        assert_eq!(subscription.devices, devices);
    }
}

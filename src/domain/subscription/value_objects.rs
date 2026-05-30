use crate::domain::error::DomainError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionPlan {
    Trial,
    Month1,
    Month3,
    Month6,
    Month12,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubscriptionStatus {
    Active,
    Inactive,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionDevices(pub i32);

// =============================================================================================

impl Display for SubscriptionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SubscriptionDevices {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl SubscriptionPlan {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionPlan::Trial => "trial",
            SubscriptionPlan::Month1 => "month_1",
            SubscriptionPlan::Month3 => "month_3",
            SubscriptionPlan::Month6 => "month_6",
            SubscriptionPlan::Month12 => "month_12",
        }
    }
}

impl FromStr for SubscriptionPlan {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "trial" => Ok(SubscriptionPlan::Trial),
            "month_1" => Ok(SubscriptionPlan::Month1),
            "month_3" => Ok(SubscriptionPlan::Month3),
            "month_6" => Ok(SubscriptionPlan::Month6),
            "month_12" => Ok(SubscriptionPlan::Month12),
            _ => Err(DomainError::InvalidPlan(s.to_string())),
        }
    }
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Inactive => "inactive",
            SubscriptionStatus::Canceled => "canceled",
        }
    }
}

impl FromStr for SubscriptionStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(SubscriptionStatus::Active),
            "inactive" => Ok(SubscriptionStatus::Inactive),
            "canceled" => Ok(SubscriptionStatus::Canceled),
            _ => Err(DomainError::InvalidStatus(s.to_string())),
        }
    }
}

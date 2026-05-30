use crate::application::error::AppResult;
use crate::domain::error::DomainError;
use crate::domain::subscription::repository::DynSubscriptionRepository;
use crate::domain::user::entity::User;

pub struct MenuState {
    pub can_trial: bool,
}

pub struct GetMenuStateQuery {
    sub_repo: DynSubscriptionRepository,
}

impl GetMenuStateQuery {
    pub fn new(sub_repo: DynSubscriptionRepository) -> Self {
        Self { sub_repo }
    }

    pub async fn execute(&self, user: &User) -> AppResult<MenuState> {
        if user.trial_used {
            return Ok(MenuState { can_trial: false });
        }

        let user_id = user.user_id.ok_or(DomainError::EntityNotSaved)?;
        let has_active_sub = self.sub_repo.find_active_by_user_id(user_id).await?.is_some();

        Ok(MenuState {
            can_trial: !has_active_sub,
        })
    }
}
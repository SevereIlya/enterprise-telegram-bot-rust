pub use crate::adapters::telegram::handlers::callback::*;

#[derive(Debug, Clone)]
pub enum CallbackAction {
    Menu(MenuAction),
    // Purchase(PurchaseAction),
    // Vpn(VpnAction),
    // Admin(AdminAction),
    // Support(SupportAction),
    Ignore,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    StartTrial,
    Router,
    Profile,
    Tariffs,
    Referral,
    Help,
    Down,
    Main,
}

impl CallbackAction {
    pub fn parse(data: &str) -> Self {
        if data == "ignore" {
            return Self::Ignore;
        }

        let mut parts = data.split(':');

        let domain = parts.next().unwrap_or("");

        match domain {
            "menu" => MenuAction::parse(&mut parts).map(Self::Menu),
            _ => None,
        }
        .unwrap_or_else(|| Self::Unknown(data.to_string()))
    }
}

impl MenuAction {
    fn parse<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Option<Self> {
        match parts.next()? {
            "trial" => Some(Self::StartTrial),
            "router" => Some(Self::Router),
            "profile" => Some(Self::Profile),
            "tariffs" => Some(Self::Tariffs),
            "referral" => Some(Self::Referral),
            "help" => Some(Self::Help),
            "down" => Some(Self::Down),
            "main" => Some(Self::Main),
            _ => None,
        }
    }
}
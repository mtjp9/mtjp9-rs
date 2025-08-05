use rand::{distr::Alphanumeric, Rng};

pub mod dbconnections;
pub mod domain;
pub mod error;
pub mod oauth;
pub mod organizations;
pub mod tickets;
pub mod token;
pub mod users;

use crate::domain::Domain;
use crate::token::BearerToken;

pub struct Auth0ClientSettings {
    pub domain: Domain,
    pub token: BearerToken,
}

/// Generate a cryptographically secure random password.
pub fn random_password() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

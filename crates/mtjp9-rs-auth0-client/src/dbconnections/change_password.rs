//! Change Password API helper
//!
//! This module wraps the **Auth0 Authentication API – Change Password** endpoint
//! (<https://auth0.com/docs/api/authentication#change-password>).
//!
//! # Example
//! ```ignore
//! use your_crate::dbconnections::{ChangePasswordRequest, change_password};
//! use your_crate::domain::Domain;
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let domain = Domain::new(env::var("AUTH0_DOMAIN")?)?;
//!
//!     let req = ChangePasswordRequest::builder()
//!         .client_id("your_client_id")
//!         .email("user@example.com")
//!         .connection("Username-Password-Authentication")
//!         .build()?;
//!
//!     let response = change_password(&domain, req).await?;
//!     println!("Password change initiated: {}", response);
//!     Ok(())
//! }
//! ```
use crate::{
    domain::Domain,
    error::{Auth0Error, Result},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    /// The client ID for your application.
    pub client_id: String,

    /// The user's email address.
    pub email: String,

    /// The name of the database connection configured for your application.
    pub connection: String,

    /// The organization ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
}

impl ChangePasswordRequest {
    pub fn builder() -> ChangePasswordRequestBuilder {
        ChangePasswordRequestBuilder::default()
    }
}

#[derive(Default)]
pub struct ChangePasswordRequestBuilder {
    client_id: Option<String>,
    email: Option<String>,
    connection: Option<String>,
    organization: Option<String>,
}

impl ChangePasswordRequestBuilder {
    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn connection(mut self, connection: impl Into<String>) -> Self {
        self.connection = Some(connection.into());
        self
    }

    pub fn organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    pub fn build(self) -> Result<ChangePasswordRequest> {
        let client_id = self
            .client_id
            .ok_or_else(|| Auth0Error::InvalidRequest("Client ID is required".to_string()))?;

        let email = self
            .email
            .ok_or_else(|| Auth0Error::InvalidRequest("Email is required".to_string()))?;

        if !email.contains('@') {
            return Err(Auth0Error::InvalidRequest(
                "Invalid email format".to_string(),
            ));
        }

        let connection = self
            .connection
            .ok_or_else(|| Auth0Error::InvalidRequest("Connection is required".to_string()))?;

        Ok(ChangePasswordRequest {
            client_id,
            email,
            connection,
            organization: self.organization,
        })
    }
}

/// The response from a successful change password request is typically
/// a plain text message like "We've just sent you an email to reset your password."
pub type ChangePasswordResponse = String;

/// Call the Auth0 Authentication API to initiate a password change.
///
/// This will send a password reset email to the user.
///
/// * `domain` – The Auth0 domain (e.g. `my-tenant.eu.auth0.com`).
/// * `request` – Body describing the password change request.
pub async fn change_password(
    domain: &Domain,
    request: ChangePasswordRequest,
) -> Result<ChangePasswordResponse> {
    let url = domain.to_url("/dbconnections/change_password");

    let resp = Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if resp.status().is_success() {
        let message = resp.text().await?;
        Ok(message)
    } else {
        Err(Auth0Error::from_response(resp).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_password_request_builder_valid() {
        let req = ChangePasswordRequest::builder()
            .client_id("test_client_id")
            .email("user@example.com")
            .connection("Username-Password-Authentication")
            .organization("org_123")
            .build();

        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.client_id, "test_client_id");
        assert_eq!(req.email, "user@example.com");
        assert_eq!(req.connection, "Username-Password-Authentication");
        assert_eq!(req.organization, Some("org_123".to_string()));
    }

    #[test]
    fn test_change_password_request_builder_minimal() {
        let req = ChangePasswordRequest::builder()
            .client_id("test_client_id")
            .email("user@example.com")
            .connection("Username-Password-Authentication")
            .build();

        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.client_id, "test_client_id");
        assert_eq!(req.email, "user@example.com");
        assert_eq!(req.connection, "Username-Password-Authentication");
        assert_eq!(req.organization, None);
    }

    #[test]
    fn test_change_password_request_builder_missing_client_id() {
        let req = ChangePasswordRequest::builder()
            .email("user@example.com")
            .connection("Username-Password-Authentication")
            .build();

        assert!(req.is_err());
        match req {
            Err(Auth0Error::InvalidRequest(msg)) => {
                assert_eq!(msg, "Client ID is required");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_change_password_request_builder_missing_email() {
        let req = ChangePasswordRequest::builder()
            .client_id("test_client_id")
            .connection("Username-Password-Authentication")
            .build();

        assert!(req.is_err());
        match req {
            Err(Auth0Error::InvalidRequest(msg)) => {
                assert_eq!(msg, "Email is required");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_change_password_request_builder_missing_connection() {
        let req = ChangePasswordRequest::builder()
            .client_id("test_client_id")
            .email("user@example.com")
            .build();

        assert!(req.is_err());
        match req {
            Err(Auth0Error::InvalidRequest(msg)) => {
                assert_eq!(msg, "Connection is required");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_change_password_request_builder_invalid_email() {
        let req = ChangePasswordRequest::builder()
            .client_id("test_client_id")
            .email("invalid_email")
            .connection("Username-Password-Authentication")
            .build();

        assert!(req.is_err());
        match req {
            Err(Auth0Error::InvalidRequest(msg)) => {
                assert_eq!(msg, "Invalid email format");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }
}

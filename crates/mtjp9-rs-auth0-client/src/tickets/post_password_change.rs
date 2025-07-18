//! Create Password Change Ticket API helper
//!
//! This module wraps the **Auth0 Management API v2 – Create Password Change Ticket** endpoint
//! (<https://auth0.com/docs/api/management/v2/tickets/post-password-change>).
//!
//! # Example
//! ```ignore
//! use your_crate::tickets::{CreatePasswordChangeTicketRequest, create_password_change_ticket};
//! use your_crate::token::BearerToken;
//! use your_crate::domain::Domain;
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let token = BearerToken::new(env::var("MGMT_API_TOKEN")?)?;
//!     let domain = Domain::new(env::var("AUTH0_DOMAIN")?)?;
//!
//!     let req = CreatePasswordChangeTicketRequest::builder()
//!         .user_id("auth0|507f1f77bcf86cd799439011")
//!         .result_url("https://myapp.com/password-changed")
//!         .ttl_sec(432000) // 5 days
//!         .mark_email_as_verified(true)
//!         .build()?;
//!
//!     let ticket = create_password_change_ticket(&domain, &token, req).await?;
//!     println!("Password change ticket URL: {}", ticket.ticket);
//!     Ok(())
//! }
//! ```
use crate::{
    domain::Domain,
    error::{Auth0Error, Result},
    token::BearerToken,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreatePasswordChangeTicketRequest {
    /// The user ID for whom the password change ticket is being created.
    pub user_id: String,

    /// The URL to redirect to after the password change is completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_url: Option<String>,

    /// The time-to-live in seconds for the ticket. Default is 432000 seconds (5 days).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_sec: Option<i32>,

    /// Whether to mark the user's email as verified when the password is changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_email_as_verified: Option<bool>,

    /// Whether to include the email address in the email (used for invitations).
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "includeEmailInRedirect"
    )]
    pub include_email_in_redirect: Option<bool>,

    /// A new email address to set for the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_email: Option<String>,

    /// The connection to use for the password change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,

    /// The client ID to use for the password change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// The organization ID to use for the password change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
}

impl CreatePasswordChangeTicketRequest {
    pub fn builder() -> CreatePasswordChangeTicketRequestBuilder {
        CreatePasswordChangeTicketRequestBuilder::default()
    }
}

#[derive(Default)]
pub struct CreatePasswordChangeTicketRequestBuilder {
    user_id: Option<String>,
    result_url: Option<String>,
    ttl_sec: Option<i32>,
    mark_email_as_verified: Option<bool>,
    include_email_in_redirect: Option<bool>,
    new_email: Option<String>,
    connection_id: Option<String>,
    client_id: Option<String>,
    organization_id: Option<String>,
}

impl CreatePasswordChangeTicketRequestBuilder {
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn result_url(mut self, result_url: impl Into<String>) -> Self {
        self.result_url = Some(result_url.into());
        self
    }

    pub fn ttl_sec(mut self, ttl_sec: i32) -> Self {
        self.ttl_sec = Some(ttl_sec);
        self
    }

    pub fn mark_email_as_verified(mut self, verified: bool) -> Self {
        self.mark_email_as_verified = Some(verified);
        self
    }

    pub fn include_email_in_redirect(mut self, include: bool) -> Self {
        self.include_email_in_redirect = Some(include);
        self
    }

    pub fn new_email(mut self, email: impl Into<String>) -> Self {
        self.new_email = Some(email.into());
        self
    }

    pub fn connection_id(mut self, connection_id: impl Into<String>) -> Self {
        self.connection_id = Some(connection_id.into());
        self
    }

    pub fn client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    pub fn organization_id(mut self, organization_id: impl Into<String>) -> Self {
        self.organization_id = Some(organization_id.into());
        self
    }

    pub fn build(self) -> Result<CreatePasswordChangeTicketRequest> {
        let user_id = self
            .user_id
            .ok_or_else(|| Auth0Error::InvalidRequest("User ID is required".to_string()))?;

        if let Some(ref email) = self.new_email {
            if !email.contains('@') {
                return Err(Auth0Error::InvalidRequest(
                    "Invalid email format".to_string(),
                ));
            }
        }

        if let Some(ttl) = self.ttl_sec {
            if ttl <= 0 {
                return Err(Auth0Error::InvalidRequest(
                    "TTL must be positive".to_string(),
                ));
            }
        }

        Ok(CreatePasswordChangeTicketRequest {
            user_id,
            result_url: self.result_url,
            ttl_sec: self.ttl_sec,
            mark_email_as_verified: self.mark_email_as_verified,
            include_email_in_redirect: self.include_email_in_redirect,
            new_email: self.new_email,
            connection_id: self.connection_id,
            client_id: self.client_id,
            organization_id: self.organization_id,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePasswordChangeTicketResponse {
    /// The URL for the password change ticket.
    pub ticket: String,
}

/// Call the Auth0 Management API to create a password change ticket.
///
/// * `domain` – The Auth0 domain (e.g. `my-tenant.eu.auth0.com`).
/// * `token` – Bearer token with `create:user_tickets` scope.
/// * `request` – Body describing the password change ticket.
pub async fn create_password_change_ticket(
    domain: &Domain,
    token: &BearerToken,
    request: CreatePasswordChangeTicketRequest,
) -> Result<CreatePasswordChangeTicketResponse> {
    let url = domain.to_url("/api/v2/tickets/password-change");

    let resp = Client::new()
        .post(url)
        .bearer_auth(token.as_str())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if resp.status().is_success() {
        let ticket = resp.json::<CreatePasswordChangeTicketResponse>().await?;
        Ok(ticket)
    } else {
        Err(Auth0Error::from_response(resp).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_password_change_ticket_request_builder_valid() {
        let req = CreatePasswordChangeTicketRequest::builder()
            .user_id("auth0|507f1f77bcf86cd799439011")
            .result_url("https://myapp.com/password-changed")
            .ttl_sec(432000)
            .mark_email_as_verified(true)
            .include_email_in_redirect(false)
            .new_email("newemail@example.com")
            .connection_id("con_123")
            .client_id("client_456")
            .organization_id("org_789")
            .build();

        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.user_id, "auth0|507f1f77bcf86cd799439011");
        assert_eq!(
            req.result_url,
            Some("https://myapp.com/password-changed".to_string())
        );
        assert_eq!(req.ttl_sec, Some(432000));
        assert_eq!(req.mark_email_as_verified, Some(true));
        assert_eq!(req.include_email_in_redirect, Some(false));
        assert_eq!(req.new_email, Some("newemail@example.com".to_string()));
        assert_eq!(req.connection_id, Some("con_123".to_string()));
        assert_eq!(req.client_id, Some("client_456".to_string()));
        assert_eq!(req.organization_id, Some("org_789".to_string()));
    }

    #[test]
    fn test_create_password_change_ticket_request_builder_minimal() {
        let req = CreatePasswordChangeTicketRequest::builder()
            .user_id("auth0|507f1f77bcf86cd799439011")
            .build();

        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.user_id, "auth0|507f1f77bcf86cd799439011");
        assert_eq!(req.result_url, None);
        assert_eq!(req.ttl_sec, None);
        assert_eq!(req.mark_email_as_verified, None);
    }

    #[test]
    fn test_create_password_change_ticket_request_builder_missing_user_id() {
        let req = CreatePasswordChangeTicketRequest::builder()
            .result_url("https://myapp.com/password-changed")
            .build();

        assert!(req.is_err());
        match req {
            Err(Auth0Error::InvalidRequest(msg)) => {
                assert_eq!(msg, "User ID is required");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_create_password_change_ticket_request_builder_invalid_email() {
        let req = CreatePasswordChangeTicketRequest::builder()
            .user_id("auth0|507f1f77bcf86cd799439011")
            .new_email("invalid_email")
            .build();

        assert!(req.is_err());
        match req {
            Err(Auth0Error::InvalidRequest(msg)) => {
                assert_eq!(msg, "Invalid email format");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_create_password_change_ticket_request_builder_invalid_ttl() {
        let req = CreatePasswordChangeTicketRequest::builder()
            .user_id("auth0|507f1f77bcf86cd799439011")
            .ttl_sec(-1)
            .build();

        assert!(req.is_err());
        match req {
            Err(Auth0Error::InvalidRequest(msg)) => {
                assert_eq!(msg, "TTL must be positive");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }
}

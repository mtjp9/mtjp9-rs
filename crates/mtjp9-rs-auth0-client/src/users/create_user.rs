//! Create User API helper
//!
//! This module wraps the **Auth0 Management API v2 – Create User** endpoint
//! (<https://auth0.com/docs/api/management/v2/users/post-users>).
//!
//! # Example
//! ```ignore
//! use your_crate::users::{CreateUserRequest, create_user};
//! use your_crate::token::BearerToken;
//! use your_crate::domain::Domain;
//! use std::env;
//!
// ! #[tokio::main]
// ! async fn main() -> anyhow::Result<()> {
// !     let token = BearerToken::new(env::var("MGMT_API_TOKEN")?)?;
// !     let domain = Domain::new(env::var("AUTH0_DOMAIN")?)?;
// !
// !     let req = CreateUserRequest::builder()
// !         .email("user@example.com")
// !         .connection("Username-Password-Authentication")
// !         .password("SecureP@ssword123!")
// !         .given_name("John")
// !         .family_name("Doe")
// !         .build()?;
// !
// !     let user = create_user(&domain, &token, req).await?;
// !     println!("Created user: {} (id = {})", user.email, user.user_id);
// !     Ok(())
// ! }
//! ```
use crate::{
    domain::Domain,
    error::{Auth0Error, Result},
    token::BearerToken,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// The user's email address.
    pub email: String,

    /// The connection to create the user in (e.g. "Username-Password-Authentication").
    pub connection: String,

    /// The user's password, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// The user's given name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    /// The user's family name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,

    /// The user's full name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The user's nickname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,

    /// URL pointing to the user's picture.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,

    /// Whether the user's id is verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Whether the user's email is verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,

    /// The user's phone number (E.164 format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,

    /// Whether the user's phone number is verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_verified: Option<bool>,

    /// Additional metadata for the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_metadata: Option<Value>,

    /// App-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_metadata: Option<Value>,

    /// Whether the user is blocked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked: Option<bool>,
}

impl CreateUserRequest {
    pub fn builder() -> CreateUserRequestBuilder {
        CreateUserRequestBuilder::default()
    }
}

#[derive(Default)]
pub struct CreateUserRequestBuilder {
    email: Option<String>,
    connection: Option<String>,
    password: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    name: Option<String>,
    nickname: Option<String>,
    picture: Option<String>,
    user_id: Option<String>,
    email_verified: Option<bool>,
    phone_number: Option<String>,
    phone_verified: Option<bool>,
    user_metadata: Option<Value>,
    app_metadata: Option<Value>,
    blocked: Option<bool>,
}

impl CreateUserRequestBuilder {
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn connection(mut self, connection: impl Into<String>) -> Self {
        self.connection = Some(connection.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn given_name(mut self, given_name: impl Into<String>) -> Self {
        self.given_name = Some(given_name.into());
        self
    }

    pub fn family_name(mut self, family_name: impl Into<String>) -> Self {
        self.family_name = Some(family_name.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nickname = Some(nickname.into());
        self
    }

    pub fn picture(mut self, picture: impl Into<String>) -> Self {
        self.picture = Some(picture.into());
        self
    }

    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn email_verified(mut self, verified: bool) -> Self {
        self.email_verified = Some(verified);
        self
    }

    pub fn phone_number(mut self, phone_number: impl Into<String>) -> Self {
        self.phone_number = Some(phone_number.into());
        self
    }

    pub fn phone_verified(mut self, verified: bool) -> Self {
        self.phone_verified = Some(verified);
        self
    }

    pub fn user_metadata(mut self, metadata: Value) -> Self {
        self.user_metadata = Some(metadata);
        self
    }

    pub fn app_metadata(mut self, metadata: Value) -> Self {
        self.app_metadata = Some(metadata);
        self
    }

    pub fn blocked(mut self, blocked: bool) -> Self {
        self.blocked = Some(blocked);
        self
    }

    pub fn build(self) -> Result<CreateUserRequest> {
        let email = self
            .email
            .ok_or_else(|| Auth0Error::InvalidRequest("Email is required".to_string()))?;

        let connection = self
            .connection
            .ok_or_else(|| Auth0Error::InvalidRequest("Connection is required".to_string()))?;

        if !email.contains('@') {
            return Err(Auth0Error::InvalidRequest(
                "Invalid email format".to_string(),
            ));
        }

        Ok(CreateUserRequest {
            email,
            connection,
            password: self.password,
            given_name: self.given_name,
            family_name: self.family_name,
            name: self.name,
            nickname: self.nickname,
            picture: self.picture,
            user_id: self.user_id,
            email_verified: self.email_verified,
            phone_number: self.phone_number,
            phone_verified: self.phone_verified,
            user_metadata: self.user_metadata,
            app_metadata: self.app_metadata,
            blocked: self.blocked,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserResponse {
    /// The user's unique identifier.
    pub user_id: String,

    /// The user's email address.
    pub email: String,

    /// Whether the user's email is verified.
    pub email_verified: bool,

    /// The user's given name.
    pub given_name: Option<String>,

    /// The user's family name.
    pub family_name: Option<String>,

    /// The user's full name.
    pub name: Option<String>,

    /// The user's nickname.
    pub nickname: Option<String>,

    /// URL pointing to the user's picture.
    pub picture: Option<String>,

    /// The user's phone number.
    pub phone_number: Option<String>,

    /// Whether the user's phone number is verified.
    pub phone_verified: Option<bool>,

    /// Additional metadata for the user.
    pub user_metadata: Option<Value>,

    /// App-specific metadata.
    pub app_metadata: Option<Value>,

    /// Whether the user is blocked.
    pub blocked: Option<bool>,

    /// When the user was created.
    pub created_at: String,

    /// When the user was last updated.
    pub updated_at: String,

    /// List of identity providers.
    pub identities: Vec<Identity>,
}

#[derive(Debug, Deserialize)]
pub struct Identity {
    /// The connection name.
    pub connection: String,

    /// The user ID for this identity.
    pub user_id: String,

    /// The identity provider.
    pub provider: String,

    /// Whether this is a social identity.
    pub is_social: Option<bool>,
}

/// Call the Auth0 Management API to create a new user.
///
/// * `domain` – The Auth0 domain (e.g. `my-tenant.eu.auth0.com`).
/// * `token` – Bearer token with `create:users` scope.
/// * `request` – Body describing the user.
pub async fn create_user(
    domain: &Domain,
    token: &BearerToken,
    request: CreateUserRequest,
) -> Result<CreateUserResponse> {
    let url = domain.to_url("/api/v2/users");

    let resp = Client::new()
        .post(url)
        .bearer_auth(token.as_str())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if resp.status().is_success() {
        let user = resp.json::<CreateUserResponse>().await?;
        Ok(user)
    } else {
        Err(Auth0Error::from_response(resp).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::random_password;
    // use crate::domain::Domain;
    // use crate::token::BearerToken;
    // use mockito::{Server, ServerGuard};
    use serde_json::json;

    // async fn setup_mock_server() -> ServerGuard {
    //     Server::new_async().await
    // }

    // fn setup_domain(server: &ServerGuard) -> Domain {
    //     let url = server.url();
    //     let domain = url.replace("http://", "").replace("https://", "");
    //     Domain::new(domain).unwrap()
    // }

    // fn setup_token() -> BearerToken {
    //     BearerToken::new("test_token_123").unwrap()
    // }

    #[test]
    fn test_create_user_request_builder_valid() {
        let req = CreateUserRequest::builder()
            .email("test@example.com")
            .connection("Username-Password-Authentication")
            .password("SecurePassword123!")
            .given_name("John")
            .family_name("Doe")
            .name("John Doe")
            .nickname("johndoe")
            .picture("https://example.com/avatar.jpg")
            .email_verified(true)
            .phone_number("+1234567890")
            .phone_verified(false)
            .user_metadata(json!({"favorite_color": "blue"}))
            .app_metadata(json!({"roles": ["user"]}))
            .blocked(false)
            .build();

        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.connection, "Username-Password-Authentication");
        assert_eq!(req.password, Some("SecurePassword123!".to_string()));
        assert_eq!(req.given_name, Some("John".to_string()));
        assert_eq!(req.family_name, Some("Doe".to_string()));
        assert_eq!(req.name, Some("John Doe".to_string()));
        assert_eq!(req.nickname, Some("johndoe".to_string()));
        assert_eq!(
            req.picture,
            Some("https://example.com/avatar.jpg".to_string())
        );
        assert_eq!(req.email_verified, Some(true));
        assert_eq!(req.phone_number, Some("+1234567890".to_string()));
        assert_eq!(req.phone_verified, Some(false));
        assert_eq!(req.user_metadata, Some(json!({"favorite_color": "blue"})));
        assert_eq!(req.app_metadata, Some(json!({"roles": ["user"]})));
        assert_eq!(req.blocked, Some(false));
    }

    #[test]
    fn test_create_user_request_builder_minimal() {
        let req = CreateUserRequest::builder()
            .email("test@example.com")
            .connection("Username-Password-Authentication")
            .build();

        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.connection, "Username-Password-Authentication");
        assert_eq!(req.password, None);
        assert_eq!(req.given_name, None);
        assert_eq!(req.family_name, None);
    }

    #[test]
    fn test_create_user_request_builder_missing_email() {
        let req = CreateUserRequest::builder()
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
    fn test_create_user_request_builder_missing_connection() {
        let req = CreateUserRequest::builder()
            .email("test@example.com")
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
    fn test_create_user_request_builder_invalid_email() {
        let req = CreateUserRequest::builder()
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

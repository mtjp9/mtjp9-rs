//! Organization creation functionality for Auth0 Management API v2
//!
//! This module provides the `create_organization` function for creating new organizations
//! in Auth0. It wraps the POST /api/v2/organizations endpoint.
//!
//! # Example
//!
//! ```no_run
//! use mtjp9_rs_auth0_client::{
//!     domain::Domain,
//!     token::BearerToken,
//!     organizations::{CreateOrganizationRequest, create_organization},
//!     error::Auth0Error,
//! };
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Auth0Error> {
//!     // Setup domain and token
//!     let domain = Domain::new(env::var("AUTH0_DOMAIN").map_err(|_| Auth0Error::InvalidRequest("AUTH0_DOMAIN not set".to_string()))?)?;
//!     let token = BearerToken::new(env::var("AUTH0_MGMT_API_TOKEN").map_err(|_| Auth0Error::InvalidRequest("AUTH0_MGMT_API_TOKEN not set".to_string()))?)?;
//!     
//!     let request = CreateOrganizationRequest {
//!         name: "acme-corp".to_string(),
//!         display_name: Some("Acme Corporation".to_string()),
//!         ..Default::default()
//!     };
//!     
//!     let organization = create_organization(&domain, &token, request).await?;
//!     println!("Created organization: {} (ID: {})", organization.name, organization.id);
//!     
//!     Ok(())
//! }
//! ```
//!
//! # API Documentation
//!
//! See the [Auth0 API documentation](https://auth0.com/docs/api/management/v2/organizations/post-organizations)
//! for more details about the organization creation endpoint.

use crate::{
    domain::Domain,
    error::{Auth0Error, Result},
    token::BearerToken,
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Default timeout for HTTP requests
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Request body for creating a new organization
///
/// See: <https://auth0.com/docs/api/management/v2/organizations/post-organizations>
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateOrganizationRequest {
    /// A unique identifier for the organization (e.g., "org-finance").
    /// This will be used to generate the organization's slug.
    pub name: String,

    /// A friendly name for the organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Branding settings for the organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding: Option<OrganizationBranding>,

    /// Metadata associated with the organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Enabled connections for the organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_connections: Option<Vec<EnabledConnection>>,
}

/// Branding configuration for an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationBranding {
    /// URL for the organization's logo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,

    /// Primary color for the organization's branding (hex format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<BrandingColors>,
}

/// Color configuration for organization branding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingColors {
    /// Primary color (hex format, e.g., "#FF5733")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,

    /// Page background color (hex format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_background: Option<String>,
}

/// Enabled connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledConnection {
    /// The connection ID
    pub connection_id: String,

    /// Whether to assign membership on login
    #[serde(default)]
    pub assign_membership_on_login: bool,

    /// Whether to show the connection as a button
    #[serde(default)]
    pub show_as_button: bool,

    #[serde(default)]
    pub is_signup_enabled: bool,
}

/// Response from creating or fetching an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationResponse {
    /// The organization's unique identifier
    pub id: String,

    /// The organization's name (slug)
    pub name: String,

    /// The organization's display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Branding settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding: Option<OrganizationBranding>,

    /// Organization metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Enabled connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_connections: Option<Vec<EnabledConnection>>,
}

/// Creates a new organization in Auth0.
///
/// This function calls the Auth0 Management API v2 to create a new organization
/// with the specified configuration.
///
/// # Arguments
///
/// * `domain` - The Auth0 domain to use for the API request
/// * `token` - A valid Management API access token with the `create:organizations` scope
/// * `request` - The organization configuration including name, display name, branding, etc.
///
/// # Errors
///
/// Returns an `Auth0Error` if:
/// * The request fails due to network issues
/// * The API returns an error response (4xx or 5xx status codes)
/// * The response cannot be deserialized
///
/// # Rate Limiting
///
/// Auth0 enforces rate limits on Management API endpoints. If you receive a 429 error,
/// implement exponential backoff before retrying.
pub async fn create_organization(
    domain: &Domain,
    token: &BearerToken,
    request: CreateOrganizationRequest,
) -> Result<OrganizationResponse> {
    // Construct the API endpoint URL using the provided domain
    let endpoint = domain.to_url("/api/v2/organizations");

    // Build the HTTP client with timeout
    let client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;

    // Send the POST request to create the organization
    let response = client
        .post(&endpoint)
        .bearer_auth(token.as_str())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    // Extract the status code before consuming the response
    let status = response.status();

    // Handle the response based on status code
    match status {
        StatusCode::CREATED | StatusCode::OK => {
            // Successfully created the organization
            response
                .json::<OrganizationResponse>()
                .await
                .map_err(Auth0Error::from)
        }
        _ => {
            // Convert error response to Auth0Error
            Err(Auth0Error::from_response(response).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::Domain, token::BearerToken};
    use mockito::Server;

    #[tokio::test]
    async fn test_create_organization_success() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = CreateOrganizationRequest {
            name: "test-org".to_string(),
            display_name: Some("Test Organization".to_string()),
            ..Default::default()
        };

        let response_body = r#"{
            "id": "org_123456",
            "name": "test-org",
            "display_name": "Test Organization"
        }"#;

        let mock = server
            .mock("POST", "/api/v2/organizations")
            .match_header("Authorization", "Bearer test-token")
            .match_header("Content-Type", "application/json")
            .match_body(mockito::Matcher::JsonString(
                serde_json::to_string(&request).unwrap(),
            ))
            .with_status(201)
            .with_header("Content-Type", "application/json")
            .with_body(response_body)
            .create_async()
            .await;

        let result = create_organization(&domain, &token, request).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let org = result.unwrap();
        assert_eq!(org.id, "org_123456");
        assert_eq!(org.name, "test-org");
        assert_eq!(org.display_name, Some("Test Organization".to_string()));
    }

    #[tokio::test]
    async fn test_create_organization_with_full_request() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = CreateOrganizationRequest {
            name: "test-org-full".to_string(),
            display_name: Some("Test Full Organization".to_string()),
            branding: Some(OrganizationBranding {
                logo_url: Some("https://example.com/logo.png".to_string()),
                colors: Some(BrandingColors {
                    primary: Some("#FF5733".to_string()),
                    page_background: Some("#FFFFFF".to_string()),
                }),
            }),
            metadata: Some(serde_json::json!({
                "industry": "technology",
                "size": "medium"
            })),
            enabled_connections: Some(vec![EnabledConnection {
                connection_id: "con_123".to_string(),
                assign_membership_on_login: true,
                show_as_button: true,
                is_signup_enabled: false,
            }]),
        };

        let response_body = serde_json::json!({
            "id": "org_full_123",
            "name": "test-org-full",
            "display_name": "Test Full Organization",
            "branding": {
                "logo_url": "https://example.com/logo.png",
                "colors": {
                    "primary": "#FF5733",
                    "page_background": "#FFFFFF"
                }
            },
            "metadata": {
                "industry": "technology",
                "size": "medium"
            },
            "enabled_connections": [{
                "connection_id": "con_123",
                "assign_membership_on_login": true,
                "show_as_button": true
            }]
        });

        let mock = server
            .mock("POST", "/api/v2/organizations")
            .with_status(201)
            .with_header("Content-Type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let result = create_organization(&domain, &token, request).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let org = result.unwrap();
        assert_eq!(org.id, "org_full_123");
        assert!(org.branding.is_some());
        assert!(org.metadata.is_some());
        assert!(org.enabled_connections.is_some());
    }

    #[tokio::test]
    async fn test_create_organization_unauthorized() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("invalid-token").expect("Valid test token");

        let request = CreateOrganizationRequest {
            name: "test-org".to_string(),
            ..Default::default()
        };

        let error_body = r#"{
            "statusCode": 401,
            "error": "Unauthorized",
            "message": "Invalid token"
        }"#;

        let mock = server
            .mock("POST", "/api/v2/organizations")
            .with_status(401)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = create_organization(&domain, &token, request).await;
        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::Unauthorized(msg) => {
                assert!(msg.contains("Invalid token"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_create_organization_rate_limited() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = CreateOrganizationRequest {
            name: "test-org".to_string(),
            ..Default::default()
        };

        let error_body = r#"{
            "statusCode": 429,
            "error": "Too Many Requests",
            "message": "Rate limit exceeded"
        }"#;

        let mock = server
            .mock("POST", "/api/v2/organizations")
            .with_status(429)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = create_organization(&domain, &token, request).await;
        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::TooManyRequests(msg) => {
                assert!(msg.contains("Rate limit exceeded"));
            }
            _ => panic!("Expected TooManyRequests error"),
        }
    }

    #[tokio::test]
    async fn test_create_organization_conflict() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = CreateOrganizationRequest {
            name: "existing-org".to_string(),
            ..Default::default()
        };

        let error_body = r#"{
            "statusCode": 409,
            "error": "Conflict",
            "message": "Organization with this name already exists"
        }"#;

        let mock = server
            .mock("POST", "/api/v2/organizations")
            .with_status(409)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = create_organization(&domain, &token, request).await;
        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::Conflict { status, body } => {
                assert_eq!(status, 409);
                assert!(body.contains("Organization with this name already exists"));
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[tokio::test]
    async fn test_create_organization_invalid_json_response() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = CreateOrganizationRequest {
            name: "test-org".to_string(),
            ..Default::default()
        };

        let mock = server
            .mock("POST", "/api/v2/organizations")
            .with_status(201)
            .with_header("Content-Type", "application/json")
            .with_body("invalid json")
            .create_async()
            .await;

        let result = create_organization(&domain, &token, request).await;
        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::Transport(_) => {
                // JSON parsing error is wrapped as Transport error
            }
            _ => panic!("Expected Transport error for invalid JSON"),
        }
    }

    #[test]
    fn test_create_organization_request_serialization() {
        let request = CreateOrganizationRequest {
            name: "test-org".to_string(),
            display_name: Some("Test Org".to_string()),
            branding: None,
            metadata: None,
            enabled_connections: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "test-org");
        assert_eq!(json["display_name"], "Test Org");
        assert!(!json.as_object().unwrap().contains_key("branding"));
        assert!(!json.as_object().unwrap().contains_key("metadata"));
        assert!(!json
            .as_object()
            .unwrap()
            .contains_key("enabled_connections"));
    }

    #[test]
    fn test_organization_response_deserialization() {
        let json = r#"{
            "id": "org_123",
            "name": "test-org",
            "display_name": "Test Organization",
            "branding": {
                "logo_url": "https://example.com/logo.png"
            }
        }"#;

        let response: OrganizationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "org_123");
        assert_eq!(response.name, "test-org");
        assert_eq!(response.display_name, Some("Test Organization".to_string()));
        assert!(response.branding.is_some());
        assert_eq!(
            response.branding.unwrap().logo_url,
            Some("https://example.com/logo.png".to_string())
        );
    }
}

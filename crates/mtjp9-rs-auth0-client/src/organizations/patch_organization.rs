//! Organization patching functionality for Auth0 Management API v2
//!
//! This module provides the `patch_organization` function for updating existing organizations
//! in Auth0. It wraps the PATCH /api/v2/organizations/{id} endpoint.
//!
//! # Example
//!
//! ```no_run
//! use mtjp9_rs_auth0_client::{
//!     domain::Domain,
//!     token::BearerToken,
//!     organizations::{PatchOrganizationRequest, patch_organization},
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
//!     let request = PatchOrganizationRequest {
//!         display_name: Some("Updated Acme Corporation".to_string()),
//!         ..Default::default()
//!     };
//!     
//!     let organization = patch_organization(&domain, &token, "org_123456", request).await?;
//!     println!("Updated organization: {} (ID: {})", organization.name, organization.id);
//!     
//!     Ok(())
//! }
//! ```
//!
//! # API Documentation
//!
//! See the [Auth0 API documentation](https://auth0.com/docs/api/management/v2/organizations/patch-organizations-by-id)
//! for more details about the organization patching endpoint.

use crate::{
    domain::Domain,
    error::{Auth0Error, Result},
    token::BearerToken,
};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// Import shared types from create_organization module
#[allow(unused_imports)]
use super::create_organization::{
    BrandingColors, EnabledConnection, OrganizationBranding, OrganizationResponse,
};

// Default timeout for HTTP requests
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Request body for patching an existing organization
///
/// All fields are optional. Only fields that are provided will be updated.
///
/// See: <https://auth0.com/docs/api/management/v2/organizations/patch-organizations-by-id>
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchOrganizationRequest {
    /// A friendly name for the organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The organization's name (slug)
    /// Note: This field might not be updatable in all Auth0 configurations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

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

/// Updates an existing organization in Auth0.
///
/// This function calls the Auth0 Management API v2 to update an existing organization
/// with the specified configuration. Only fields provided in the request will be updated.
///
/// # Arguments
///
/// * `domain` - The Auth0 domain to use for the API request
/// * `token` - A valid Management API access token with the `update:organizations` scope
/// * `organization_id` - The ID of the organization to update (e.g., "org_123456")
/// * `request` - The organization fields to update
///
/// # Errors
///
/// Returns an `Auth0Error` if:
/// * The request fails due to network issues
/// * The API returns an error response (4xx or 5xx status codes)
/// * The response cannot be deserialized
/// * The organization ID is not found
///
/// # Rate Limiting
///
/// Auth0 enforces rate limits on Management API endpoints. If you receive a 429 error,
/// implement exponential backoff before retrying.
pub async fn patch_organization(
    domain: &Domain,
    token: &BearerToken,
    organization_id: &str,
    request: PatchOrganizationRequest,
) -> Result<OrganizationResponse> {
    // Construct the API endpoint URL using the provided domain and organization ID
    let endpoint = domain.to_url(&format!("/api/v2/organizations/{organization_id}"));

    // Build the HTTP client with timeout
    let client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;

    // Send the PATCH request to update the organization
    let response = client
        .patch(&endpoint)
        .bearer_auth(token.as_str())
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    // Extract the status code before consuming the response
    let status = response.status();

    // Handle the response based on status code
    match status {
        StatusCode::OK => {
            // Successfully updated the organization
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
    async fn test_patch_organization_success() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = PatchOrganizationRequest {
            display_name: Some("Updated Test Organization".to_string()),
            ..Default::default()
        };

        let response_body = r#"{
            "id": "org_123456",
            "name": "test-org",
            "display_name": "Updated Test Organization"
        }"#;

        let mock = server
            .mock("PATCH", "/api/v2/organizations/org_123456")
            .match_header("Authorization", "Bearer test-token")
            .match_header("Content-Type", "application/json")
            .match_body(mockito::Matcher::JsonString(
                serde_json::to_string(&request).unwrap(),
            ))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(response_body)
            .create_async()
            .await;

        let result = patch_organization(&domain, &token, "org_123456", request).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let org = result.unwrap();
        assert_eq!(org.id, "org_123456");
        assert_eq!(org.name, "test-org");
        assert_eq!(
            org.display_name,
            Some("Updated Test Organization".to_string())
        );
    }

    #[tokio::test]
    async fn test_patch_organization_with_full_request() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = PatchOrganizationRequest {
            display_name: Some("Updated Full Organization".to_string()),
            branding: Some(OrganizationBranding {
                logo_url: Some("https://example.com/new-logo.png".to_string()),
                colors: Some(BrandingColors {
                    primary: Some("#00FF00".to_string()),
                    page_background: Some("#F0F0F0".to_string()),
                }),
            }),
            metadata: Some(serde_json::json!({
                "industry": "fintech",
                "size": "large",
                "updated": true
            })),
            enabled_connections: Some(vec![EnabledConnection {
                connection_id: "con_456".to_string(),
                assign_membership_on_login: false,
                show_as_button: true,
                is_signup_enabled: false,
            }]),
            ..Default::default()
        };

        let response_body = serde_json::json!({
            "id": "org_full_123",
            "name": "test-org-full",
            "display_name": "Updated Full Organization",
            "branding": {
                "logo_url": "https://example.com/new-logo.png",
                "colors": {
                    "primary": "#00FF00",
                    "page_background": "#F0F0F0"
                }
            },
            "metadata": {
                "industry": "fintech",
                "size": "large",
                "updated": true
            },
            "enabled_connections": [{
                "connection_id": "con_456",
                "assign_membership_on_login": false,
                "show_as_button": true
            }]
        });

        let mock = server
            .mock("PATCH", "/api/v2/organizations/org_full_123")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(response_body.to_string())
            .create_async()
            .await;

        let result = patch_organization(&domain, &token, "org_full_123", request).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let org = result.unwrap();
        assert_eq!(org.id, "org_full_123");
        assert!(org.branding.is_some());
        assert!(org.metadata.is_some());
        assert!(org.enabled_connections.is_some());
    }

    #[tokio::test]
    async fn test_patch_organization_not_found() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = PatchOrganizationRequest {
            display_name: Some("Updated Organization".to_string()),
            ..Default::default()
        };

        let error_body = r#"{
            "statusCode": 404,
            "error": "Not Found",
            "message": "Organization not found"
        }"#;

        let mock = server
            .mock("PATCH", "/api/v2/organizations/org_nonexistent")
            .with_status(404)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = patch_organization(&domain, &token, "org_nonexistent", request).await;
        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::UnexpectedResponse { status, body } => {
                assert_eq!(status, 404);
                assert!(body.contains("Organization not found"));
            }
            _ => panic!("Expected UnexpectedResponse error with 404 status"),
        }
    }

    #[tokio::test]
    async fn test_patch_organization_unauthorized() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("invalid-token").expect("Valid test token");

        let request = PatchOrganizationRequest {
            display_name: Some("Updated Organization".to_string()),
            ..Default::default()
        };

        let error_body = r#"{
            "statusCode": 401,
            "error": "Unauthorized",
            "message": "Invalid token"
        }"#;

        let mock = server
            .mock("PATCH", "/api/v2/organizations/org_123456")
            .with_status(401)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = patch_organization(&domain, &token, "org_123456", request).await;
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
    async fn test_patch_organization_rate_limited() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = PatchOrganizationRequest {
            display_name: Some("Updated Organization".to_string()),
            ..Default::default()
        };

        let error_body = r#"{
            "statusCode": 429,
            "error": "Too Many Requests",
            "message": "Rate limit exceeded"
        }"#;

        let mock = server
            .mock("PATCH", "/api/v2/organizations/org_123456")
            .with_status(429)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = patch_organization(&domain, &token, "org_123456", request).await;
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
    async fn test_patch_organization_bad_request() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = PatchOrganizationRequest {
            metadata: Some(serde_json::json!({
                "invalid_field": "value"
            })),
            ..Default::default()
        };

        let error_body = r#"{
            "statusCode": 400,
            "error": "Bad Request",
            "message": "Invalid metadata field"
        }"#;

        let mock = server
            .mock("PATCH", "/api/v2/organizations/org_123456")
            .with_status(400)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = patch_organization(&domain, &token, "org_123456", request).await;
        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::InvalidRequest(msg) => {
                assert!(msg.contains("Invalid metadata field"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[tokio::test]
    async fn test_patch_organization_empty_request() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = PatchOrganizationRequest::default();

        let response_body = r#"{
            "id": "org_123456",
            "name": "test-org",
            "display_name": "Test Organization"
        }"#;

        let mock = server
            .mock("PATCH", "/api/v2/organizations/org_123456")
            .match_body(mockito::Matcher::JsonString("{}".to_string()))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(response_body)
            .create_async()
            .await;

        let result = patch_organization(&domain, &token, "org_123456", request).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        // Empty request should still return the organization
        let org = result.unwrap();
        assert_eq!(org.id, "org_123456");
    }

    #[test]
    fn test_patch_organization_request_serialization() {
        let request = PatchOrganizationRequest {
            display_name: Some("Updated Org".to_string()),
            branding: Some(OrganizationBranding {
                logo_url: Some("https://example.com/logo.png".to_string()),
                colors: None,
            }),
            metadata: None,
            enabled_connections: None,
            name: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["display_name"], "Updated Org");
        assert_eq!(json["branding"]["logo_url"], "https://example.com/logo.png");
        assert!(!json.as_object().unwrap().contains_key("metadata"));
        assert!(!json.as_object().unwrap().contains_key("name"));
        assert!(!json
            .as_object()
            .unwrap()
            .contains_key("enabled_connections"));
    }

    #[test]
    fn test_patch_organization_request_empty_serialization() {
        let request = PatchOrganizationRequest::default();
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json, serde_json::json!({}));
    }
}

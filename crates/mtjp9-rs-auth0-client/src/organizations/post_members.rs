//! Add members to an organization
//!
//! This module provides the `post_members` function for adding members to an organization
//! in Auth0. It wraps the POST /api/v2/organizations/{id}/members endpoint.
//!
//! # Example
//!
//! ```no_run
//! use mtjp9_rs_auth0_client::{
//!     domain::Domain,
//!     token::BearerToken,
//!     organizations::{AddMembersRequest, post_members},
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
//!     let request = AddMembersRequest {
//!         members: vec![
//!             "auth0|507f1f77bcf86cd799439011".to_string(),
//!             "auth0|507f1f77bcf86cd799439012".to_string(),
//!         ],
//!     };
//!     
//!     post_members(&domain, &token, "org_123456", request).await?;
//!     println!("Members added successfully");
//!     
//!     Ok(())
//! }
//! ```
//!
//! # API Documentation
//!
//! See the [Auth0 API documentation](https://auth0.com/docs/api/management/v2/organizations/post-members)
//! for more details about the add members endpoint.

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

/// Request body for adding members to an organization
///
/// See: <https://auth0.com/docs/api/management/v2/organizations/post-members>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMembersRequest {
    /// An array of user IDs to add to the organization as members
    pub members: Vec<String>,
}

/// Adds members to an organization in Auth0.
///
/// This function calls the Auth0 Management API v2 to add one or more users
/// as members of the specified organization.
///
/// # Arguments
///
/// * `domain` - The Auth0 domain to use for the API request
/// * `token` - A valid Management API access token with the `create:organization_members` scope
/// * `organization_id` - The ID of the organization to add members to
/// * `request` - The request containing user IDs to add as members
///
/// # Errors
///
/// Returns an `Auth0Error` if:
/// * The request fails due to network issues
/// * The API returns an error response (4xx or 5xx status codes)
/// * The organization ID is invalid
/// * Any of the user IDs don't exist
///
/// # Rate Limiting
///
/// Auth0 enforces rate limits on Management API endpoints. If you receive a 429 error,
/// implement exponential backoff before retrying.
pub async fn post_members(
    domain: &Domain,
    token: &BearerToken,
    organization_id: &str,
    request: AddMembersRequest,
) -> Result<()> {
    // Validate organization_id
    if organization_id.is_empty() {
        return Err(Auth0Error::InvalidRequest(
            "Organization ID cannot be empty".to_string(),
        ));
    }

    // Validate that members array is not empty
    if request.members.is_empty() {
        return Err(Auth0Error::InvalidRequest(
            "Members array cannot be empty".to_string(),
        ));
    }

    // Construct the API endpoint URL using the provided domain
    let endpoint = domain.to_url(&format!("/api/v2/organizations/{organization_id}/members"));

    // Build the HTTP client with timeout
    let client = Client::builder().timeout(REQUEST_TIMEOUT).build()?;

    // Send the POST request to add members
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
        StatusCode::NO_CONTENT | StatusCode::OK | StatusCode::CREATED => {
            // Successfully added members
            Ok(())
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
    async fn test_post_members_success() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = AddMembersRequest {
            members: vec![
                "auth0|507f1f77bcf86cd799439011".to_string(),
                "auth0|507f1f77bcf86cd799439012".to_string(),
            ],
        };

        let mock = server
            .mock("POST", "/api/v2/organizations/org_123456/members")
            .match_header("Authorization", "Bearer test-token")
            .match_header("Content-Type", "application/json")
            .match_body(mockito::Matcher::JsonString(
                serde_json::to_string(&request).unwrap(),
            ))
            .with_status(204)
            .create_async()
            .await;

        let result = post_members(&domain, &token, "org_123456", request).await;
        mock.assert_async().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_post_members_empty_organization_id() {
        let domain = Domain::new("https://test.auth0.com").expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = AddMembersRequest {
            members: vec!["auth0|507f1f77bcf86cd799439011".to_string()],
        };

        let result = post_members(&domain, &token, "", request).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::InvalidRequest(msg) => {
                assert_eq!(msg, "Organization ID cannot be empty");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[tokio::test]
    async fn test_post_members_empty_members_array() {
        let domain = Domain::new("https://test.auth0.com").expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = AddMembersRequest { members: vec![] };

        let result = post_members(&domain, &token, "org_123456", request).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::InvalidRequest(msg) => {
                assert_eq!(msg, "Members array cannot be empty");
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[tokio::test]
    async fn test_post_members_organization_not_found() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = AddMembersRequest {
            members: vec!["auth0|507f1f77bcf86cd799439011".to_string()],
        };

        let error_body = r#"{
            "statusCode": 404,
            "error": "Not Found",
            "message": "Organization not found"
        }"#;

        let mock = server
            .mock("POST", "/api/v2/organizations/org_invalid/members")
            .with_status(404)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = post_members(&domain, &token, "org_invalid", request).await;
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
    async fn test_post_members_unauthorized() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("invalid-token").expect("Valid test token");

        let request = AddMembersRequest {
            members: vec!["auth0|507f1f77bcf86cd799439011".to_string()],
        };

        let error_body = r#"{
            "statusCode": 401,
            "error": "Unauthorized",
            "message": "Invalid token"
        }"#;

        let mock = server
            .mock("POST", "/api/v2/organizations/org_123456/members")
            .with_status(401)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = post_members(&domain, &token, "org_123456", request).await;
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
    async fn test_post_members_bad_request() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).expect("Valid test domain");
        let token = BearerToken::new("test-token").expect("Valid test token");

        let request = AddMembersRequest {
            members: vec!["invalid_user_id".to_string()],
        };

        let error_body = r#"{
            "statusCode": 400,
            "error": "Bad Request",
            "message": "Invalid user ID format"
        }"#;

        let mock = server
            .mock("POST", "/api/v2/organizations/org_123456/members")
            .with_status(400)
            .with_header("Content-Type", "application/json")
            .with_body(error_body)
            .create_async()
            .await;

        let result = post_members(&domain, &token, "org_123456", request).await;
        mock.assert_async().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Auth0Error::InvalidRequest(msg) => {
                assert!(msg.contains("Invalid user ID format"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_add_members_request_serialization() {
        let request = AddMembersRequest {
            members: vec![
                "auth0|507f1f77bcf86cd799439011".to_string(),
                "auth0|507f1f77bcf86cd799439012".to_string(),
            ],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["members"].is_array());
        assert_eq!(json["members"][0], "auth0|507f1f77bcf86cd799439011");
        assert_eq!(json["members"][1], "auth0|507f1f77bcf86cd799439012");
    }
}

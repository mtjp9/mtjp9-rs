use crate::{domain::Domain, error::Auth0Error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Request body for Auth0's OAuth 2.0 token endpoint.
///
/// This struct supports multiple OAuth 2.0 grant types:
/// - `client_credentials`: For machine-to-machine authentication
/// - `authorization_code`: For web applications with user login
/// - `refresh_token`: For refreshing access tokens
///
/// # Examples
///
/// ## Client Credentials Grant
/// ```ignore
/// let request = OauthTokenRequest {
///     grant_type: "client_credentials".to_string(),
///     client_id: "your_client_id".to_string(),
///     client_secret: Some("your_client_secret".to_string()),
///     audience: Some("https://your-api.example.com".to_string()),
///     ..Default::default()
/// };
/// ```
///
/// ## Authorization Code Grant
/// ```ignore
/// let request = OauthTokenRequest {
///     grant_type: "authorization_code".to_string(),
///     client_id: "your_client_id".to_string(),
///     client_secret: Some("your_client_secret".to_string()),
///     code: Some("authorization_code_from_callback".to_string()),
///     redirect_uri: Some("https://your-app.com/callback".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Serialize, Default)]
pub struct OauthTokenRequest {
    /// The type of grant being requested.
    /// Common values: "client_credentials", "authorization_code", "refresh_token"
    pub grant_type: String,

    /// Your application's Client ID
    pub client_id: String,

    /// Your application's Client Secret (required for confidential clients)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// The unique identifier of the target API (required for client_credentials grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,

    /// The authorization code received from the authorization endpoint (for authorization_code grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The redirect URI that was used in the authorization request (for authorization_code grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,

    /// The PKCE code verifier (for authorization_code grant with PKCE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,

    /// The refresh token (for refresh_token grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// Space-separated list of scopes to request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Successful response from Auth0's `/oauth/token` endpoint.
///
/// Contains the access token and related metadata returned after successful authentication.
#[derive(Debug, Deserialize)]
pub struct OauthTokenResponse {
    /// The access token that can be used to call Auth0 APIs
    pub access_token: String,

    /// The scopes granted for this access token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// The type of token (typically "Bearer")
    pub token_type: String,

    /// The number of seconds until the access token expires
    pub expires_in: u64,

    /// The refresh token (only returned for certain grant types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// The ID token (only returned when using OpenID Connect scopes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
}

/// Error response from Auth0's `/oauth/token` endpoint.
///
/// Auth0 returns standardized OAuth 2.0 error responses when token requests fail.
#[derive(Debug, Deserialize)]
pub struct OauthErrorResponse {
    /// The error code (e.g., "invalid_request", "invalid_grant", "unauthorized_client")
    pub error: String,

    /// Human-readable description of the error
    pub error_description: String,

    /// A URI identifying a human-readable web page with error information
    #[allow(dead_code)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

/// Request an access token from Auth0's OAuth 2.0 token endpoint.
///
/// This function supports multiple OAuth 2.0 grant types including:
/// - Client Credentials: For machine-to-machine authentication
/// - Authorization Code: For web applications with user login
/// - Refresh Token: For refreshing expired access tokens
///
/// # Arguments
///
/// * `domain` - The Auth0 domain (e.g., "tenant.auth0.com")
/// * `request` - The token request containing grant type and related parameters
///
/// # Returns
///
/// Returns `Ok(OauthTokenResponse)` on success, containing the access token and metadata.
/// Returns `Err` on failure, with detailed error information.
///
/// # Errors
///
/// This function will return an error if:
/// - Network request fails (connection, DNS, TLS issues)
/// - Auth0 returns a non-2xx status code
/// - Response cannot be deserialized
///
/// # Examples
///
/// ## Client Credentials Grant
/// ```ignore
/// let domain = Domain::new("tenant.auth0.com")?;
/// let request = OauthTokenRequest {
///     grant_type: "client_credentials".to_string(),
///     client_id: "your_client_id".to_string(),
///     client_secret: Some("your_client_secret".to_string()),
///     audience: Some("https://your-api.example.com".to_string()),
///     ..Default::default()
/// };
/// let token = get_oauth_token(&domain, request).await?;
/// ```
///
/// ## Authorization Code Grant with PKCE
/// ```ignore
/// let domain = Domain::new("tenant.auth0.com")?;
/// let request = OauthTokenRequest {
///     grant_type: "authorization_code".to_string(),
///     client_id: "your_client_id".to_string(),
///     code: Some("authorization_code".to_string()),
///     redirect_uri: Some("https://your-app.com/callback".to_string()),
///     code_verifier: Some("your_code_verifier".to_string()),
///     ..Default::default()
/// };
/// let token = get_oauth_token(&domain, request).await?;
/// ```
pub async fn get_oauth_token(
    domain: &Domain,
    request: OauthTokenRequest,
) -> Result<OauthTokenResponse, Auth0Error> {
    let url = domain.to_url("/oauth/token");

    let response = Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(Auth0Error::from)?;

    // Check if the response is successful
    if response.status().is_success() {
        response
            .json::<OauthTokenResponse>()
            .await
            .map_err(Auth0Error::from)
    } else {
        // Try to parse error response
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        // Attempt to parse as OAuth error response
        if let Ok(oauth_error) = serde_json::from_str::<OauthErrorResponse>(&body) {
            match oauth_error.error.as_str() {
                "invalid_request" => Err(Auth0Error::InvalidRequest(oauth_error.error_description)),
                "unauthorized_client" | "invalid_client" => {
                    Err(Auth0Error::Unauthorized(oauth_error.error_description))
                }
                "access_denied" | "insufficient_scope" => {
                    Err(Auth0Error::Forbidden(oauth_error.error_description))
                }
                _ => Err(Auth0Error::UnexpectedResponse {
                    status: status.as_u16(),
                    body: format!("{}: {}", oauth_error.error, oauth_error.error_description),
                }),
            }
        } else {
            // Fallback to generic error handling
            match status.as_u16() {
                400 => Err(Auth0Error::InvalidRequest(body)),
                401 => Err(Auth0Error::Unauthorized(body)),
                403 => Err(Auth0Error::Forbidden(body)),
                429 => Err(Auth0Error::TooManyRequests(body)),
                _ => Err(Auth0Error::UnexpectedResponse {
                    status: status.as_u16(),
                    body,
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_client_credentials_success() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).unwrap();

        let mock = server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token": "test_access_token",
                "token_type": "Bearer",
                "expires_in": 86400,
                "scope": "read:users"
            }"#,
            )
            .create_async()
            .await;

        let request = OauthTokenRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "test_client_id".to_string(),
            client_secret: Some("test_client_secret".to_string()),
            audience: Some("https://api.example.com".to_string()),
            ..Default::default()
        };

        let result = get_oauth_token(&domain, request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.access_token, "test_access_token");
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 86400);
        assert_eq!(response.scope, Some("read:users".to_string()));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_authorization_code_with_refresh_token() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).unwrap();

        let mock = server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token": "test_access_token",
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": "test_refresh_token",
                "id_token": "test_id_token"
            }"#,
            )
            .create_async()
            .await;

        let request = OauthTokenRequest {
            grant_type: "authorization_code".to_string(),
            client_id: "test_client_id".to_string(),
            client_secret: Some("test_client_secret".to_string()),
            code: Some("test_auth_code".to_string()),
            redirect_uri: Some("https://app.example.com/callback".to_string()),
            ..Default::default()
        };

        let result = get_oauth_token(&domain, request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.access_token, "test_access_token");
        assert_eq!(
            response.refresh_token,
            Some("test_refresh_token".to_string())
        );
        assert_eq!(response.id_token, Some("test_id_token".to_string()));

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_oauth_error_invalid_request() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).unwrap();

        let mock = server
            .mock("POST", "/oauth/token")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "error": "invalid_request",
                "error_description": "Missing required parameter: client_id"
            }"#,
            )
            .create_async()
            .await;

        let request = OauthTokenRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "".to_string(),
            ..Default::default()
        };

        let result = get_oauth_token(&domain, request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            Auth0Error::InvalidRequest(msg) => {
                assert_eq!(msg, "Missing required parameter: client_id");
            }
            _ => panic!("Expected InvalidRequest error"),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_oauth_error_unauthorized() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).unwrap();

        let mock = server
            .mock("POST", "/oauth/token")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "error": "invalid_client",
                "error_description": "Client authentication failed"
            }"#,
            )
            .create_async()
            .await;

        let request = OauthTokenRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "test_client_id".to_string(),
            client_secret: Some("wrong_secret".to_string()),
            ..Default::default()
        };

        let result = get_oauth_token(&domain, request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            Auth0Error::Unauthorized(msg) => {
                assert_eq!(msg, "Client authentication failed");
            }
            _ => panic!("Expected Unauthorized error"),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_non_json_error_response() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).unwrap();

        let mock = server
            .mock("POST", "/oauth/token")
            .with_status(400)
            .with_header("content-type", "text/plain")
            .with_body("Bad Request")
            .create_async()
            .await;

        let request = OauthTokenRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "test_client_id".to_string(),
            ..Default::default()
        };

        let result = get_oauth_token(&domain, request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            Auth0Error::InvalidRequest(msg) => {
                assert_eq!(msg, "Bad Request");
            }
            _ => panic!("Expected InvalidRequest error"),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_rate_limit_error() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).unwrap();

        let mock = server
            .mock("POST", "/oauth/token")
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Too Many Requests"}"#)
            .create_async()
            .await;

        let request = OauthTokenRequest {
            grant_type: "client_credentials".to_string(),
            client_id: "test_client_id".to_string(),
            ..Default::default()
        };

        let result = get_oauth_token(&domain, request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            Auth0Error::TooManyRequests(_) => {}
            _ => panic!("Expected TooManyRequests error"),
        }

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_pkce_authorization_code() {
        let mut server = Server::new_async().await;
        let domain = Domain::new(server.url()).unwrap();

        let mock = server
            .mock("POST", "/oauth/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token": "test_access_token",
                "token_type": "Bearer",
                "expires_in": 3600
            }"#,
            )
            .create_async()
            .await;

        let request = OauthTokenRequest {
            grant_type: "authorization_code".to_string(),
            client_id: "test_client_id".to_string(),
            code: Some("test_auth_code".to_string()),
            redirect_uri: Some("https://app.example.com/callback".to_string()),
            code_verifier: Some("test_verifier".to_string()),
            ..Default::default()
        };

        let result = get_oauth_token(&domain, request).await;
        assert!(result.is_ok());

        mock.assert_async().await;
    }
}

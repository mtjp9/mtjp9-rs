//! Create Organization API helper
//!
//! This module wraps the **Auth0 Management API v2 – Create Organization** endpoint
//! (<https://auth0.com/docs/api/management/v2/organizations/post-organizations>). It
//! mirrors the style of the `get_oauth_token` helper already in the code‑base.
//!
//! # Example
//! ```ignore
//! use your_crate::organization::{CreateOrganizationRequest, create_organization};
//! use std::env;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let token = env::var("MGMT_API_TOKEN")?; // management‑API access token
//!     let req = CreateOrganizationRequest { name: "acme".into(), ..Default::default() };
//!     let org = create_organization(&token, req).await?;
//!     println!("Created organization: {} (id = {})", org.name, org.id);
//!     Ok(())
//! }
//! ```

use crate::{
    error::Auth0Error,
    pb::{CreateOrganizationRequest, OrganizationResponse},
};
use reqwest::Client;

/* =========================
 * Endpoint helper
 * =======================*/

/// Call the Auth0 Management API to create a new organisation.
///
/// * `token` – Bearer token with `create:organizations` scope.
/// * `request` – Body describing the organisation.
///
/// Environment variables used:
/// * `AUTH0_DOMAIN` – e.g. `my‑tenant.eu.auth0.com` (no protocol, no trailing slash).
pub async fn create_organization(
    token: &str,
    request: CreateOrganizationRequest,
) -> Result<OrganizationResponse, Auth0Error> {
    let domain = std::env::var("AUTH0_DOMAIN").expect("Missing AUTH0_DOMAIN env var");
    let url = format!("https://{}/api/v2/organizations", domain);

    let resp = Client::new()
        .post(url)
        .bearer_auth(token)
        .json(&request)
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(resp.json::<OrganizationResponse>().await?)
    } else {
        Err(Auth0Error::from_response(resp).await)
    }
}

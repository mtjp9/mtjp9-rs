//! Organization management functionality for Auth0
//!
//! This module provides functions for managing organizations through the Auth0 Management API v2.

mod create_organization;
mod patch_organization;

pub use create_organization::{
    create_organization, CreateOrganizationRequest, OrganizationResponse,
};
pub use patch_organization::{patch_organization, PatchOrganizationRequest};

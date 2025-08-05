//! Organization management functionality for Auth0
//!
//! This module provides functions for managing organizations through the Auth0 Management API v2.

mod create_organization;
mod patch_organization;
mod post_members;

pub use create_organization::{
    create_organization, BrandingColors, CreateOrganizationRequest, EnabledConnection,
    OrganizationBranding, OrganizationResponse,
};
pub use patch_organization::{patch_organization, PatchOrganizationRequest};
pub use post_members::{post_members, AddMembersRequest};

use crate::error::{Auth0Error, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    inner: String,
}

impl Domain {
    pub fn new(domain: impl Into<String>) -> Result<Self> {
        let domain = domain.into();

        if domain.is_empty() {
            return Err(Auth0Error::InvalidRequest(
                "Domain cannot be empty".to_string(),
            ));
        }

        // Allow http:// prefix for testing with mockito
        #[cfg(not(test))]
        if domain.starts_with("http://") || domain.starts_with("https://") {
            return Err(Auth0Error::InvalidRequest(
                "Domain should not include protocol (http:// or https://)".to_string(),
            ));
        }

        #[cfg(test)]
        if domain.starts_with("https://") {
            return Err(Auth0Error::InvalidRequest(
                "Domain should not include protocol (https://)".to_string(),
            ));
        }

        if domain.ends_with('/') {
            return Err(Auth0Error::InvalidRequest(
                "Domain should not end with a trailing slash".to_string(),
            ));
        }

        // Allow domains without dots for testing purposes (e.g., localhost:1234)
        #[cfg(not(test))]
        if !domain.contains('.') {
            return Err(Auth0Error::InvalidRequest(
                "Domain must be a valid Auth0 domain (e.g., tenant.auth0.com)".to_string(),
            ));
        }

        Ok(Self { inner: domain })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn to_url(&self, path: &str) -> String {
        // For testing with mockito, allow http:// prefixed domains
        if self.inner.starts_with("http://") {
            format!("{}{}", self.inner, path)
        } else {
            format!("https://{}{}", self.inner, path)
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl TryFrom<String> for Domain {
    type Error = Auth0Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Domain {
    type Error = Auth0Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_domain() {
        let domain = Domain::new("tenant.auth0.com").unwrap();
        assert_eq!(domain.as_str(), "tenant.auth0.com");
        assert_eq!(
            domain.to_url("/api/v2/users"),
            "https://tenant.auth0.com/api/v2/users"
        );
    }

    #[test]
    fn test_invalid_domains() {
        assert!(Domain::new("").is_err());
        assert!(Domain::new("https://tenant.auth0.com").is_err());
        assert!(Domain::new("tenant.auth0.com/").is_err());
    }
}

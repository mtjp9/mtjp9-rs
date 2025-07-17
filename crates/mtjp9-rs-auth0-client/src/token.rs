use crate::error::{Auth0Error, Result};
use std::fmt;

#[derive(Clone)]
pub struct BearerToken {
    inner: String,
}

impl BearerToken {
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let token = token.into();

        if token.is_empty() {
            return Err(Auth0Error::InvalidRequest(
                "Bearer token cannot be empty".to_string(),
            ));
        }

        if token.contains(char::is_whitespace) {
            return Err(Auth0Error::InvalidRequest(
                "Bearer token cannot contain whitespace".to_string(),
            ));
        }

        Ok(Self { inner: token })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl fmt::Debug for BearerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BearerToken")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

impl fmt::Display for BearerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl TryFrom<String> for BearerToken {
    type Error = Auth0Error;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for BearerToken {
    type Error = Auth0Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_token() {
        let token = BearerToken::new("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9").unwrap();
        assert_eq!(token.as_str(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_invalid_tokens() {
        assert!(BearerToken::new("").is_err());
        assert!(BearerToken::new("token with spaces").is_err());
        assert!(BearerToken::new("token\nwith\nnewlines").is_err());
    }

    #[test]
    fn test_token_debug_redacted() {
        let token = BearerToken::new("secret_token").unwrap();
        let debug_str = format!("{token:?}");
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("secret_token"));
    }
}

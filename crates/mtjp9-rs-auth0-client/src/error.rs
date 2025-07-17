pub type Result<T, E = Auth0Error> = std::result::Result<T, E>;

/// High‑level errors returned by the Auth0 helpers.
#[derive(Debug, thiserror::Error)]
pub enum Auth0Error {
    /// Low‑level transport / JSON errors.
    #[error("network/json error: {0}")]
    Transport(#[from] reqwest::Error),

    /// 400 – The request parameters are invalid.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// 401 – Any authentication failure (invalid token, not global, bad JWT sig …).
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// 403 – Caller authenticated but lacks required scopes.
    #[error("forbidden / insufficient scope: {0}")]
    Forbidden(String),

    #[error("conflict status {status}: {body}")]
    Conflict { status: u16, body: String },

    /// 429 – Too many requests (rate limited).
    #[error("rate limited: {0}")]
    TooManyRequests(String),

    /// Any other non‑success HTTP status.
    #[error("unexpected status {status}: {body}")]
    UnexpectedResponse { status: u16, body: String },
}

impl Auth0Error {
    /// Convert an HTTP response into `Auth0Error` if it isn’t a success.
    pub async fn from_response(resp: reqwest::Response) -> Self {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        match status.as_u16() {
            400 => Self::InvalidRequest(body),
            401 => Self::Unauthorized(body),
            403 => Self::Forbidden(body),
            409 => Self::Conflict {
                status: status.as_u16(),
                body,
            },
            429 => Self::TooManyRequests(body),
            code => Self::UnexpectedResponse { status: code, body },
        }
    }
}

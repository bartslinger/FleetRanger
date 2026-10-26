//! Errors of the JSON API, as HTTP responses. Messages are written for the
//! person using the web app, which shows them as they are.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub enum ApiError {
    /// The actor may not do this.
    Forbidden,
    NotFound,
    /// The request itself is wrong, e.g. an empty name.
    Invalid(String),
    /// It conflicts with what is stored, e.g. a name that is already taken.
    Conflict(String),
    Internal(anyhow::Error),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Forbidden => StatusCode::FORBIDDEN.into_response(),
            Self::NotFound => StatusCode::NOT_FOUND.into_response(),
            Self::Invalid(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::Conflict(message) => (StatusCode::CONFLICT, message).into_response(),
            Self::Internal(error) => {
                tracing::error!(error = format!("{error:#}"), "request failed");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

/// Whether a query failed on a unique constraint, e.g. a duplicate name.
pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|error| error.is_unique_violation())
}

/// An app role that doesn't exist (any more), refused by its foreign key.
pub fn unknown_app_role(error: sqlx::Error) -> ApiError {
    if error
        .as_database_error()
        .is_some_and(|error| error.is_foreign_key_violation())
    {
        ApiError::Invalid("One of the app roles no longer exists. Reload the page.".to_owned())
    } else {
        error.into()
    }
}

/// A trimmed, non-empty name.
pub fn name(value: &str) -> ApiResult<String> {
    let name = value.trim();
    if name.is_empty() {
        return Err(ApiError::Invalid("Enter a name.".to_owned()));
    }
    Ok(name.to_owned())
}

// See https://github.com/davidpdrsn/realworld-axum-sqlx/blob/main/src/http/error.rs

use aide::OperationIo;
use axum::{
    body::Body,
    http::{Response, StatusCode, header::WWW_AUTHENTICATE},
    response::IntoResponse,
};
use std::fmt::Debug;

use ApiError::*;

/// Main return type for the API.
/// See [ApiError] for details on usage.
pub type ApiResult<T> = std::result::Result<T, ApiError>;

/// Custom `Error` type for use by route handlers.
/// Errors should be meaningful are parsable by the front-end.
/// However, errors caused by problems with the back-end or database
/// should not eplain the actual cause to the front-end.
///
/// `anyhow::Error` and `sqlx::Error` types can be coerced into `ApiError` by using
/// the `?` operator or `Into::into`
#[derive(thiserror::Error, Debug, OperationIo)]
#[aide(output)]
pub enum ApiError {
    /// Returns `400 Bad Request`
    #[error("invalid request: {0}")]
    BadRequest(String),

    /// Returns `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Returns `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Returns `404 Not Found`
    #[error("request path not found")]
    NotFound,

    /// Returns `409 Conflict`
    #[error("conflict with current state: {0}")]
    Conflict(String),

    /// Returns `422 Unprocessable Entity`
    #[error("error in the request body: {0}")]
    UnprocessableEntity(String),

    /// Returns `500 Internal Server Error` on a `sqlx::Error`.
    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    /// Returns `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl PartialEq for ApiError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BadRequest(l0), Self::BadRequest(r0)) => l0 == r0,
            (Self::Conflict(l0), Self::Conflict(r0)) => l0 == r0,
            (Self::UnprocessableEntity(l0), Self::UnprocessableEntity(r0)) => l0 == r0,
            (Self::Sqlx(l0), Self::Sqlx(r0)) => l0.to_string() == r0.to_string(),
            (Self::Anyhow(l0), Self::Anyhow(r0)) => l0.to_string() == r0.to_string(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl ApiError {
    /// Maps `ApiError` variants to `StatusCode`s
    fn status_code(&self) -> StatusCode {
        match self {
            BadRequest(_) => StatusCode::BAD_REQUEST,
            Unauthorized => StatusCode::UNAUTHORIZED,
            Forbidden => StatusCode::FORBIDDEN,
            NotFound => StatusCode::NOT_FOUND,
            Conflict(_) => StatusCode::CONFLICT,
            UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Sqlx(_) | Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        match self {
            Unauthorized => {
                return (
                    self.status_code(),
                    // Include the `WWW-Authenticate` challenge required in the specification
                    // for the `401 Unauthorized` response code:
                    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401
                    [(WWW_AUTHENTICATE, "Token")],
                    self.to_string(),
                )
                    .into_response();
            }
            Sqlx(ref e) => {
                tracing::error!("SQLx error: {:?}", e);
            }
            Anyhow(ref e) => {
                tracing::error!("Anyhow error: {:?}", e);
            }
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

/// Trait used to extend `Result` to allow conversion to [anyhow::Result].
pub trait IntoAnyhow<T> {
    fn anyhow(self) -> Result<T, anyhow::Error>;
}

impl<T, E> IntoAnyhow<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn anyhow(self) -> Result<T, anyhow::Error> {
        self.map_err(anyhow::Error::from)
    }
}

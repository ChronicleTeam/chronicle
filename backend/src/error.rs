// See https://github.com/davidpdrsn/realworld-axum-sqlx/blob/main/src/http/error.rs

use axum::{
    body::Body,
    http::{header::WWW_AUTHENTICATE, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use sqlx::error::DatabaseError;
use std::{borrow::Cow, collections::HashMap};

pub type ApiResult<T> = std::result::Result<T, ApiError>;

pub struct ErrorMessage {
    pub key: Cow<'static, str>,
    pub message: Cow<'static, str>,
}

impl ErrorMessage {
    pub const fn new_static(key: &'static str, message: &'static str) -> Self {
        ErrorMessage {
            key: Cow::Borrowed(key),
            message: Cow::Borrowed(message),
        }
    }

    pub fn new<K, V>(key: K, message: V) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        ErrorMessage {
            key: key.into(),
            message: message.into(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    // Return `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Return `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Return `404 Not Found`
    #[error("request path not found")]
    NotFound,

    /// Return `422 Unprocessable Entity`
    #[error("error in the request body")]
    UnprocessableEntity(HashMap<Cow<'static, str>, Cow<'static, str>>),

    /// Automatically return `500 Internal Server Error` on a `sqlx::Error`.
    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    /// Return `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl ApiError {
    pub fn unprocessable_entity(errors: impl IntoIterator<Item = ErrorMessage>) -> Self {
        Self::UnprocessableEntity(HashMap::from_iter(
            errors.into_iter().map(|msg| (msg.key, msg.message)),
        ))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        match self {
            Self::UnprocessableEntity(errors) => {
                return (StatusCode::UNPROCESSABLE_ENTITY, Json(errors)).into_response();
            }
            Self::Unauthorized => {
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

            Self::Sqlx(ref e) => {
                tracing::error!("SQLx error: {:?}", e);
            }

            Self::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
            }

            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

pub trait OnConstraint<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> ApiError,
    ) -> Result<T, ApiError>;
}

impl<T, E> OnConstraint<T> for Result<T, E>
where
    E: Into<ApiError>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> ApiError,
    ) -> Result<T, ApiError> {
        self.map_err(|e| match e.into() {
            ApiError::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}

// See https://github.com/davidpdrsn/realworld-axum-sqlx/blob/main/src/http/error.rs

use axum::{
    body::Body,
    http::{header::WWW_AUTHENTICATE, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use sqlx::error::DatabaseError;
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Display},
};

/// Main return type for the API.
/// See [`ApiError`] for details on usage.
pub type ApiResult<T> = std::result::Result<T, ApiError>;

/// A message returned as JSON for [`ApiError::UnprocessableEntity`].
/// The `key` should refer to the offending JSON key and the `message`
/// explains the cause of the error.
pub struct ErrorMessage {
    pub key: Cow<'static, str>,
    pub message: Cow<'static, str>,
}

impl ErrorMessage {
    /// Constructor for const declarations.
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

/// Custom `Error` type for use by route handlers.
/// Errors should be meaningful are parsable by the front-end.
/// However, errors caused by problems with the back-end or database
/// should not eplain the actual cause to the front-end.
///
/// `anyhow::Error` and `sqlx::Error` types can be coerced into `ApiError` by using
/// the `?` operator or `Into::into`
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    // Returns `400 Bad Request`
    #[error("invalid request")]
    BadRequest,

    // Returns `401 Unauthorized`
    #[error("authentication required")]
    Unauthorized,

    /// Returns `403 Forbidden`
    #[error("user may not perform that action")]
    Forbidden,

    /// Returns `404 Not Found`
    #[error("request path not found")]
    NotFound,

    /// Returns `422 Unprocessable Entity`
    #[error("error in the request body")]
    UnprocessableEntity {
        errors: HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>,
    },

    /// Returns `500 Internal Server Error` on a `sqlx::Error`.
    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),

    /// Returns `500 Internal Server Error` on a `anyhow::Error`.
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl ApiError {
    /// Create an `ApiError::UnprocessableEntity` from a collection of [`ErrorMessage`]
    ///
    /// This is a convience to manually creating the error.
    pub fn unprocessable_entity<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self::UnprocessableEntity { errors: error_map }
    }

    /// Maps `ApiError` variants to `StatusCode`s
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
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
            Self::UnprocessableEntity { errors } => {
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
                tracing::error!("Anyhow error: {:?}", e);
            }

            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

/// Custom trait to map a database constraint `sqlx::Error` to an ApiError
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
    /// Maps a database contraint `sqlx::Error` to an ApiError.
    ///
    /// This is useful for checking expected database contrainst errors and returning an appropriate response.
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

pub trait IntoAnyhow<T> {
    fn into_anyhow(self) -> Result<T, anyhow::Error>;
}

impl<T, E> IntoAnyhow<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn into_anyhow(self) -> Result<T, anyhow::Error> {
        self.map_err(anyhow::Error::from)
    }
}

pub trait IntoMessage<T> {
    fn into_msg(self) -> Result<T, anyhow::Error>;
}

impl<T, E> IntoMessage<T> for Result<T, E>
where
    E: Display + Debug + Send + Sync + 'static,
{
    fn into_msg(self) -> Result<T, anyhow::Error> {
        self.map_err(anyhow::Error::msg)
    }
}

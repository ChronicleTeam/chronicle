use axum_login::AuthUser;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::Id;

/// The application user.
#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: Id,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("user_id", &self.user_id)
            .field("username", &self.username)
            .field("password_hash", &"[redacted]")
            .field("is_admin", &self.is_admin)
            .finish()
    }
}

impl AuthUser for User {
    type Id = Id;

    fn id(&self) -> Self::Id {
        self.user_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        // We use the password hash as the auth
        // hash--what this means
        // is when the user changes their password the
        // auth session becomes invalid.
        self.password_hash.as_bytes()
    }
}

/// Credentials request type.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SelectUser {
    pub user_id: Id,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password: Option<String>,
}

/// User response type.
#[derive(Debug, Serialize, FromRow)]
pub struct UserResponse {
    pub user_id: Id,
    pub username: String,
    pub is_admin: bool
}
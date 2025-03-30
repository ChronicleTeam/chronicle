use axum_login::AuthUser;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::Id;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: Id,
    pub username: String,
    pub password_hash: String,
    pub role: UserRole,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("user_id", &self.user_id)
            .field("username", &self.username)
            .field("password_hash", &"[redacted]")
            .field("role", &self.role)
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

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role")]
pub enum UserRole {
    Admin,
    Normal,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user_id: Id,
    pub username: String,
}
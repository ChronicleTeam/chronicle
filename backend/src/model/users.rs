use core::fmt;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserBody<T> {
    pub user: T,
}

#[derive(serde::Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize, Default, PartialEq, Eq)]
#[serde(default)] // fill in any missing fields with `..UpdateUser::default()`
pub struct UpdateUserModel {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: String,
    pub image: Option<String>,
}

#[derive(Debug)]
pub struct UserIdentifier {
    user_id: Uuid,
    schema: String,
}
impl UserIdentifier {
    pub fn new(user_id: Uuid, schema: &str) -> Self {
        Self {
            user_id,
            schema: schema.to_string(),
        }
    }
}
impl fmt::Display for UserIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""{}"."c{}""#, self.schema, self.user_id)
    }
}

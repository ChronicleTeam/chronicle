use serde::{Deserialize, Serialize};
use sqlx::{prelude::*, types::chrono::{DateTime, Utc}, PgPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub user_id: i32,                
    pub username: String,               
    pub password_hash: String,        
    pub created_at: DateTime<Utc>,      
    pub updated_at: Option<DateTime<Utc>>, 

}

impl User {
    pub async fn find_by_id(pool: &PgPool, user_id: i32) -> Result<Self, sqlx::Error> {
        sqlx::query_as("SELECT * FROM app_user WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await
    }
}
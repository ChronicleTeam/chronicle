use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::Id;

/// Dashboard entity
#[derive(Serialize, FromRow)]
pub struct Dashboard {
    pub dashboard_id: Id,
    pub user_id: Id,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Create dashboard request.
#[derive(Deserialize)]
pub struct CreateDashboard {
    pub name: String,
    pub description: String,
}

/// Update dashboard request.
#[derive(Deserialize)]
pub struct UpdateDashboard {
    pub name: String,
    pub description: String,
}
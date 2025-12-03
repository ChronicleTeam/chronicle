use crate::{Id, model::access::AccessRole};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// Dashboard entity
#[derive(Debug, Serialize, Deserialize, FromRow, JsonSchema, PartialEq, Eq)]
pub struct Dashboard {
    pub dashboard_id: Id,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Create dashboard request.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDashboard {
    pub name: String,
    pub description: String,
}

/// Update dashboard request.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateDashboard {
    pub name: String,
    pub description: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GetDashboard {
    #[sqlx(flatten)]
    pub dashboard: Dashboard,
    pub access_role: AccessRole,
}

#[derive(Deserialize, JsonSchema)]
pub struct SelectDashboard {
    pub dashboard_id: Id,
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{error::{ApiError, ApiResult}, Id};


#[derive(Debug, Clone, Copy, sqlx::Type, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[sqlx(type_name = "access_role")]
pub enum AccessRole {
    Viewer,
    Editor,
    Owner,
}

pub trait AccessRoleCheck {
    fn check(self, required: AccessRole) -> ApiResult<()>;
}

impl AccessRoleCheck for Option<AccessRole> {
    fn check(self, required: AccessRole) -> ApiResult<()> {
        use AccessRole::*;
        if let Some(actual) = self {
            if match actual {
                Viewer => matches!(required, Viewer),
                Editor => matches!(required, Editor | Viewer),
                Owner => true,
            } {
                Ok(())
            } else {
                Err(ApiError::Forbidden)
            }
        } else {
            Err(ApiError::NotFound)
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub enum Resource {
    Table,
    Dashboard
}

impl Resource {
    pub fn access_tablename(&self) -> &'static str {
        match self {
            Resource::Table => "meta_table_access",
            Resource::Dashboard => "dashboard_access",
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SelectResource {
    pub resource: Resource,
    pub resource_id: Id,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAccess {
    pub username: String,
    pub access_role: AccessRole,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAccess {
    pub username: String,
    pub access_role: AccessRole,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAccess {
    pub username: String,
}

#[derive(Debug, Deserialize, FromRow, JsonSchema)]
pub struct GetAccess {
    pub username: String,
    pub access_role: AccessRole,
}
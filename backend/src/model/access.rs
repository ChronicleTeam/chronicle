//! Types for access managment features.

use crate::{
    Id,
    error::{ApiError, ApiResult},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// The access role for a user and a resource.
#[derive(
    Debug,
    Clone,
    Copy,
    sqlx::Type,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
#[sqlx(type_name = "access_role")]
pub enum AccessRole {
    /// Can view the content
    Viewer,
    /// Can edit the content
    Editor,
    /// Can modify and delete the resource and its metadata
    Owner,
}

/// Trait for checking that an `Option<AccessRole>` matches the required `AccessRole`
/// and return the appropriate API response.
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

/// A resource for which a user can have access.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum Resource {
    Table,
    Dashboard,
}

impl Resource {
    /// SQL tables for the access relationship table.
    pub fn access_tablename(&self) -> &'static str {
        match self {
            Resource::Table => "meta_table_access",
            Resource::Dashboard => "dashboard_access",
        }
    }
}

/// Resource ID path extractor.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SelectResource {
    pub resource: Resource,
    pub resource_id: Id,
}

/// Create access request.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateAccess {
    pub username: String,
    pub access_role: AccessRole,
}

/// Update access request.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateAccess {
    pub username: String,
    pub access_role: AccessRole,
}

/// Delete access request.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DeleteAccess {
    pub username: String,
}

/// Get access response.
#[derive(Debug, Serialize, Deserialize, FromRow, JsonSchema, PartialEq, Eq)]
pub struct GetAccess {
    pub username: String,
    pub access_role: AccessRole,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        error::ApiError,
        model::access::{AccessRole, AccessRoleCheck},
    };

    #[test]
    fn access_role_check() {
        for (actual, required, result) in [
            (None, AccessRole::Viewer, Err(ApiError::NotFound)),
            (None, AccessRole::Editor, Err(ApiError::NotFound)),
            (None, AccessRole::Owner, Err(ApiError::NotFound)),
            (Some(AccessRole::Viewer), AccessRole::Viewer, Ok(())),
            (
                Some(AccessRole::Viewer),
                AccessRole::Editor,
                Err(ApiError::Forbidden),
            ),
            (
                Some(AccessRole::Viewer),
                AccessRole::Owner,
                Err(ApiError::Forbidden),
            ),
            (Some(AccessRole::Editor), AccessRole::Viewer, Ok(())),
            (Some(AccessRole::Editor), AccessRole::Editor, Ok(())),
            (
                Some(AccessRole::Editor),
                AccessRole::Owner,
                Err(ApiError::Forbidden),
            ),
            (Some(AccessRole::Owner), AccessRole::Viewer, Ok(())),
            (Some(AccessRole::Owner), AccessRole::Editor, Ok(())),
            (Some(AccessRole::Owner), AccessRole::Owner, Ok(())),
        ] {
            assert_eq!(actual.check(required), result);
        }
    }
}

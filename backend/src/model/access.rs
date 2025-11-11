use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{
    Id,
    error::{ApiError, ApiResult},
};

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
    Dashboard,
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

#[derive(Debug, Serialize, FromRow, JsonSchema, PartialEq, Eq)]
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

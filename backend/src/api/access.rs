use crate::{
    AppState, Id,
    api::NO_DATA_IN_REQUEST_BODY,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult},
    model::access::{
        AccessRole, AccessRoleCheck, CreateAccess, DeleteAccess, GetAccess, Resource,
        SelectResource, UpdateAccess,
    },
};
use aide::{
    NoApi,
    axum::{ApiRouter, routing::post_with},
};
use axum::{
    Json,
    extract::{Path, State},
};
use axum_login::AuthSession;
use itertools::Itertools;
use sqlx::PgConnection;

const USERNAME_NOT_FOUND: &str = "Username not found";
const USER_ALREADY_HAS_ACCESS: &str = "User already has access";
const OWNER_CANNOT_MODIFY_THEIR_OWN_ACCESS: &str = "Owner cannot modify their own access";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/{resource}/{resource_id}/access",
        post_with(create_access, docs::create_access)
            .patch_with(update_many_access, docs::update_access)
            .delete_with(delete_many_access, docs::delete_access)
            .get_with(get_all_access, docs::get_all_access),
    )
}

async fn create_access(
    State(AppState { db }): State<AppState>,
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    Path(SelectResource {
        resource,
        resource_id,
    }): Path<SelectResource>,
    Json(create_access): Json<CreateAccess>,
) -> ApiResult<()> {
    let auth_user_id = auth_user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;
    db::get_access_role(tx.as_mut(), resource, resource_id, auth_user_id)
        .await?
        .check(AccessRole::Owner)?;

    let user_id = db::get_user_by_username(tx.as_mut(), create_access.username)
        .await?
        .ok_or(ApiError::UnprocessableEntity(USERNAME_NOT_FOUND.into()))?
        .user_id;

    if let Some(_) = db::get_access_role(tx.as_mut(), resource, resource_id, user_id).await? {
        return Err(ApiError::Conflict(USER_ALREADY_HAS_ACCESS.into()));
    }

    db::create_access(
        tx.as_mut(),
        resource,
        resource_id,
        user_id,
        create_access.access_role,
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

async fn update_many_access(
    State(AppState { db }): State<AppState>,
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    Path(SelectResource {
        resource,
        resource_id,
    }): Path<SelectResource>,
    Json(update_access_vec): Json<Vec<UpdateAccess>>,
) -> ApiResult<()> {
    let auth_user_id = auth_user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;
    db::get_access_role(tx.as_mut(), resource, resource_id, auth_user_id)
        .await?
        .check(AccessRole::Owner)?;

    if update_access_vec.is_empty() {
        return Err(ApiError::BadRequest(NO_DATA_IN_REQUEST_BODY.into()));
    }
    let (usernames, access_roles): (Vec<_>, Vec<_>) = update_access_vec
        .into_iter()
        .map(|a| (a.username, a.access_role))
        .unzip();
    let user_ids =
        get_users_with_access(tx.as_mut(), auth_user_id, resource, resource_id, usernames).await?;

    db::update_many_access(
        tx.as_mut(),
        resource,
        resource_id,
        user_ids.into_iter().zip(access_roles),
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

async fn delete_many_access(
    State(AppState { db }): State<AppState>,
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    Path(SelectResource {
        resource,
        resource_id,
    }): Path<SelectResource>,
    Json(delete_access_vec): Json<Vec<DeleteAccess>>,
) -> ApiResult<()> {
    let auth_user_id = auth_user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;
    db::get_access_role(tx.as_mut(), resource, resource_id, auth_user_id)
        .await?
        .check(AccessRole::Owner)?;

    if delete_access_vec.is_empty() {
        return Err(ApiError::BadRequest(NO_DATA_IN_REQUEST_BODY.into()));
    }

    let user_ids: Vec<Id> = get_users_with_access(
        tx.as_mut(),
        auth_user_id,
        resource,
        resource_id,
        delete_access_vec.into_iter().map(|a| a.username),
    )
    .await?;

    db::delete_many_access(tx.as_mut(), resource, resource_id, user_ids).await?;

    tx.commit().await?;
    Ok(())
}

async fn get_all_access(
    State(AppState { db }): State<AppState>,
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    Path(SelectResource {
        resource,
        resource_id,
    }): Path<SelectResource>,
) -> ApiResult<Json<Vec<GetAccess>>> {
    let auth_user_id = auth_user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;
    db::get_access_role(tx.as_mut(), resource, resource_id, auth_user_id)
        .await?
        .check(AccessRole::Owner)?;

    let get_access_vec = db::get_all_access(tx.as_mut(), resource, resource_id).await?;
    tx.commit().await?;
    Ok(Json(get_access_vec))
}
async fn get_users_with_access(
    conn: &mut PgConnection,
    auth_user_id: Id,
    resource: Resource,
    resource_id: Id,
    usernames: impl IntoIterator<Item = String>,
) -> ApiResult<Vec<Id>> {
    let mut user_ids: Vec<Id> = Vec::new();
    let mut not_found_usernames: Vec<String> = Vec::new();
    for username in usernames {
        if let Some(user) = db::get_user_by_username(conn.as_mut(), username.clone()).await? {
            if user.user_id == auth_user_id {
                return Err(ApiError::UnprocessableEntity(
                    OWNER_CANNOT_MODIFY_THEIR_OWN_ACCESS.into(),
                ));
            }
            if db::get_access_role(conn.as_mut(), resource, resource_id, user.user_id)
                .await?
                .is_none()
            {
                not_found_usernames.push(username);
            }
            user_ids.push(user.user_id);
        } else {
            not_found_usernames.push(username);
        }
    }

    if !not_found_usernames.is_empty() {
        return Err(ApiError::UnprocessableEntity(format!(
            "{USERNAME_NOT_FOUND}: {}",
            not_found_usernames.into_iter().join(", ")
        )));
    }
    Ok(user_ids)
}

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs {
    use crate::{
        api::access::{USER_ALREADY_HAS_ACCESS, USERNAME_NOT_FOUND},
        docs::{ACCESS_TAG, TransformOperationExt, template},
        model::access::{AccessRole, GetAccess, Resource},
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    fn access<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, ACCESS_TAG)
            .response_description::<404, ()>("Resource not found")
            .required_access([
                (Resource::Table, AccessRole::Owner),
                (Resource::Dashboard, AccessRole::Owner),
            ])
    }

    pub fn create_access(op: TransformOperation) -> TransformOperation {
        access::<()>(
            op,
            "create_access",
            "Create a new user access to the resource.",
        )
        .response_description::<409, String>(USER_ALREADY_HAS_ACCESS)
        .response_description::<422, String>(USERNAME_NOT_FOUND)
    }

    pub fn update_access(op: TransformOperation) -> TransformOperation {
        access::<()>(
            op,
            "update_access",
            "Update a list of user access roles for the resource.",
        )
        .response_description::<422, String>(&format!("{USERNAME_NOT_FOUND}: <username>, ..."))
    }

    pub fn delete_access(op: TransformOperation) -> TransformOperation {
        access::<()>(op, "delete_access", "Delete a list of user access.")
            .response_description::<422, String>(&format!("{USERNAME_NOT_FOUND}: <username>, ..."))
    }

    pub fn get_all_access(op: TransformOperation) -> TransformOperation {
        access::<Json<Vec<GetAccess>>>(op, "get_all_access", "Get all user access to the resource.")
    }
}

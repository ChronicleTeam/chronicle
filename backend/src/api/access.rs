use crate::{
    auth::AppAuthSession, db, error::{ApiError, ApiResult}, model::access::{AccessRole, AccessRoleCheck, CreateAccess, DeleteAccess, GetAccess, SelectResource, UpdateAccess}, AppState, Id
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

const USERNAME_NOT_FOUND: &str = "Username not found";
const OWNER_CANNOT_MODIFY_THEIR_OWN_ACCESS: &str = "Owner cannot modify their own access";


pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/{resource}/{resource_id}/access",
        post_with(create_access, docs::create_access)
            .patch_with(update_access, docs::update_access).delete_with(delete_access, docs::delete_access),
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
    db::get_access(tx.as_mut(), resource, resource_id, auth_user_id)
        .await?
        .check(AccessRole::Owner)?;

    let user_id = db::get_user_by_username(tx.as_mut(), create_access.username)
        .await?
        .ok_or(ApiError::UnprocessableEntity(USERNAME_NOT_FOUND.into()))?
        .user_id;

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

async fn update_access(
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
    db::get_access(tx.as_mut(), resource, resource_id, auth_user_id)
        .await?
        .check(AccessRole::Owner)?;

    let mut user_access_roles: Vec<(Id, AccessRole)> = Vec::new();
    let mut not_found_usernames: Vec<String> = Vec::new();
    for UpdateAccess {
        username,
        access_role,
    } in update_access_vec
    {
        if let Some(user) = db::get_user_by_username(tx.as_mut(), username.clone()).await? {
            if user.user_id == auth_user_id {
                return Err(ApiError::UnprocessableEntity(
                    OWNER_CANNOT_MODIFY_THEIR_OWN_ACCESS.into(),
                ));
            }
            user_access_roles.push((user.user_id, access_role));
        } else {
            not_found_usernames.push(username);
        }
    }

    if !not_found_usernames.is_empty() {
        return Err(ApiError::UnprocessableEntity(format!(
            "Usernames not found: {}",
            not_found_usernames.into_iter().join(", ")
        )));
    }

    db::update_many_access(tx.as_mut(), resource, resource_id, user_access_roles).await?;

    tx.commit().await?;
    Ok(())
}


async fn delete_access(
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
    db::get_access(tx.as_mut(), resource, resource_id, auth_user_id)
        .await?
        .check(AccessRole::Owner)?;

    if delete_access_vec.is_empty() {
        re
    }

    let mut user_ids: Vec<Id> = Vec::new();
    let mut not_found_usernames: Vec<String> = Vec::new();
    for DeleteAccess {
        username,
    } in delete_access_vec
    {
        if let Some(user) = db::get_user_by_username(tx.as_mut(), username.clone()).await? {
            if user.user_id == auth_user_id {
                return Err(ApiError::UnprocessableEntity(
                    OWNER_CANNOT_MODIFY_THEIR_OWN_ACCESS.into(),
                ));
            }
            user_ids.push(user.user_id);
        } else {
            not_found_usernames.push(username);
        }
    }

    if !not_found_usernames.is_empty() {
        return Err(ApiError::UnprocessableEntity(format!(
            "Usernames not found: {}",
            not_found_usernames.into_iter().join(", ")
        )));
    }

    db::delete_many_access(tx.as_mut(), resource, resource_id, user_ids).await?;

    tx.commit().await?;
    Ok(())
}

mod docs {
    use crate::{
        docs::{ACCESS_TAG, TransformOperationExt, template},
    };
    use aide::{OperationOutput, transform::TransformOperation};

    fn access<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, ACCESS_TAG)
            .response_description::<403, ()>("User is not an admin")
    }

    pub fn create_access(op: TransformOperation) -> TransformOperation {
        access::<()>(op, "", "")
    }

    pub fn update_access(op: TransformOperation) -> TransformOperation {
        access::<()>(op, "", "")
    }
}

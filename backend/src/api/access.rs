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
use sqlx::{Acquire, Postgres};

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

    if db::get_access_role(tx.as_mut(), resource, resource_id, user_id)
        .await?
        .is_some()
    {
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
    println!("Ok");
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
    conn: impl Acquire<'_, Database = Postgres>,
    auth_user_id: Id,
    resource: Resource,
    resource_id: Id,
    usernames: impl IntoIterator<Item = String>,
) -> ApiResult<Vec<Id>> {
    let mut tx = conn.begin().await?;
    let mut user_ids: Vec<Id> = Vec::new();
    let mut not_found_usernames: Vec<String> = Vec::new();
    for username in usernames {
        if let Some(user) = db::get_user_by_username(tx.as_mut(), username.clone()).await? {
            if user.user_id == auth_user_id {
                return Err(ApiError::UnprocessableEntity(
                    OWNER_CANNOT_MODIFY_THEIR_OWN_ACCESS.into(),
                ));
            }
            if db::get_access_role(tx.as_mut(), resource, resource_id, user.user_id)
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
    tx.commit().await?;
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use rand::Rng;
    use serde_json::json;
    use sqlx::PgPool;

    use crate::{
        db,
        error::ApiError,
        model::{
            access::{AccessRole, CreateAccess, DeleteAccess, GetAccess, Resource, UpdateAccess},
            data::CreateTable,
            viz::CreateDashboard,
        },
        setup_tracing, test_util,
    };

    #[sqlx::test]
    async fn create_access(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let resource_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let resource = Resource::Table;

        let user = db::create_user(&db, "A".into(), "".into(), false).await?;

        let path = format!(
            "/api/{}/{resource_id}/access",
            serde_json::to_string(&resource)?.replace("\"", "")
        );

        let create_access = CreateAccess {
            username: user.username.clone(),
            access_role: AccessRole::Viewer,
        };

        server
            .post(&path)
            .json(&create_access)
            .await
            .assert_status_unauthorized();

        let auth_user = db::create_user(&db, "auth_user".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &auth_user).await;
        test_util::test_access_control(
            &db,
            resource,
            resource_id,
            auth_user.user_id,
            AccessRole::Owner,
            async || {
                let mut rng = rand::rng();
                let username =
                    db::create_user(&db, rng.random::<u64>().to_string(), "".into(), false)
                        .await
                        .unwrap()
                        .username;
                let create_access = CreateAccess {
                    username,
                    access_role: AccessRole::Viewer,
                };
                server.post(&path).json(&create_access).await
            },
        )
        .await;

        server
            .post("/api/Table/1000/access")
            .json(&create_access)
            .await
            .assert_status_not_found();

        server
            .post(&path)
            .json(&create_access)
            .await
            .assert_status_ok();
        let access_role = db::get_access_role(&db, resource, resource_id, user.user_id)
            .await?
            .unwrap();
        assert_eq!(create_access.access_role, access_role);

        server
            .post(&path)
            .json(&create_access)
            .await
            .assert_status_conflict();

        server
            .post(&path)
            .json(&CreateAccess {
                username: "wrong".into(),
                access_role: AccessRole::Viewer,
            })
            .await
            .assert_status_unprocessable_entity();
        Ok(())
    }

    #[sqlx::test]
    async fn update_access(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let resource_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let resource = Resource::Table;

        let user_1 = db::create_user(&db, "A".into(), "".into(), false).await?;
        let user_2 = db::create_user(&db, "B".into(), "".into(), false).await?;
        db::create_access(
            &db,
            resource,
            resource_id,
            user_1.user_id,
            AccessRole::Editor,
        )
        .await?;
        db::create_access(
            &db,
            resource,
            resource_id,
            user_2.user_id,
            AccessRole::Viewer,
        )
        .await?;
        let path = format!(
            "/api/{}/{resource_id}/access",
            serde_json::to_string(&resource)?.replace("\"", "")
        );

        let update_access_vec = vec![
            UpdateAccess {
                username: user_1.username.clone(),
                access_role: AccessRole::Editor,
            },
            UpdateAccess {
                username: user_2.username.clone(),
                access_role: AccessRole::Viewer,
            },
        ];
        server
            .patch(&path)
            .json(&update_access_vec)
            .await
            .assert_status_unauthorized();

        let auth_user = db::create_user(&db, "auth_user".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &auth_user).await;
        test_util::test_access_control(
            &db,
            resource,
            resource_id,
            auth_user.user_id,
            AccessRole::Owner,
            async || server.patch(&path).json(&update_access_vec).await,
        )
        .await;

        server
            .patch("/api/Table/1000/access")
            .json(&update_access_vec)
            .await
            .assert_status_not_found();

        let user_1_access_role = AccessRole::Owner;
        let user_2_access_role = AccessRole::Editor;
        let update_access_vec = vec![
            UpdateAccess {
                username: user_1.username,
                access_role: user_1_access_role,
            },
            UpdateAccess {
                username: user_2.username,
                access_role: user_2_access_role,
            },
        ];

        server
            .patch(&path)
            .json(&update_access_vec)
            .await
            .assert_status_ok();
        assert_eq!(
            db::get_access_role(&db, resource, resource_id, user_1.user_id)
                .await?
                .unwrap(),
            user_1_access_role
        );
        assert_eq!(
            db::get_access_role(&db, resource, resource_id, user_1.user_id)
                .await?
                .unwrap(),
            user_1_access_role
        );

        server
            .patch(&path)
            .json(&json!([]))
            .await
            .assert_status_bad_request();

        server
            .patch(&path)
            .json(&vec![UpdateAccess {
                username: auth_user.username.into(),
                access_role: AccessRole::Viewer,
            }])
            .await
            .assert_status_unprocessable_entity();
        Ok(())
    }

    #[sqlx::test]
    async fn delete_access(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let resource_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let resource = Resource::Table;

        let user_1 = db::create_user(&db, "A".into(), "".into(), false).await?;
        let user_2 = db::create_user(&db, "B".into(), "".into(), false).await?;
        db::create_access(
            &db,
            resource,
            resource_id,
            user_1.user_id,
            AccessRole::Editor,
        )
        .await?;
        db::create_access(
            &db,
            resource,
            resource_id,
            user_2.user_id,
            AccessRole::Viewer,
        )
        .await?;
        let path = format!(
            "/api/{}/{resource_id}/access",
            serde_json::to_string(&resource)?.replace("\"", "")
        );

        let delete_access_vec = vec![
            DeleteAccess {
                username: user_1.username.clone(),
            },
            DeleteAccess {
                username: user_2.username.clone(),
            },
        ];
        server
            .delete(&path)
            .json(&delete_access_vec)
            .await
            .assert_status_unauthorized();
        setup_tracing();
        let auth_user = db::create_user(&db, "auth_user".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &auth_user).await;
        test_util::test_access_control(
            &db,
            resource,
            resource_id,
            auth_user.user_id,
            AccessRole::Owner,
            async || {
                let mut rng = rand::rng();
                let user = db::create_user(&db, rng.random::<u64>().to_string(), "".into(), false)
                    .await
                    .unwrap();
                db::create_access(&db, resource, resource_id, user.user_id, AccessRole::Viewer)
                    .await
                    .unwrap();
                server
                    .delete(&path)
                    .json(&vec![DeleteAccess {
                        username: user.username,
                    }])
                    .await
            },
        )
        .await;

        server
            .delete("/api/Table/1000/access")
            .json(&delete_access_vec)
            .await
            .assert_status_not_found();

        server
            .delete(&path)
            .json(&delete_access_vec)
            .await
            .assert_status_ok();
        assert!(
            db::get_access_role(&db, resource, resource_id, user_1.user_id)
                .await?
                .is_none()
        );
        assert!(
            db::get_access_role(&db, resource, resource_id, user_2.user_id)
                .await?
                .is_none()
        );

        server
            .delete(&path)
            .json(&delete_access_vec)
            .await
            .assert_status_unprocessable_entity();

        server
            .delete(&path)
            .json(&json!([]))
            .await
            .assert_status_bad_request();
        Ok(())
    }

    #[sqlx::test]
    async fn get_all_access(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let resource_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let resource = Resource::Table;

        let mut user_access_1 = Vec::new();
        for (idx, access_role) in [AccessRole::Viewer, AccessRole::Editor, AccessRole::Owner]
            .into_iter()
            .enumerate()
        {
            let user = db::create_user(&db, idx.to_string(), "".into(), false).await?;
            db::create_access(&db, resource, resource_id, user.user_id, access_role).await?;
            user_access_1.push(GetAccess {
                username: user.username,
                access_role,
            });
        }

        let path = format!(
            "/api/{}/{resource_id}/access",
            serde_json::to_string(&resource)?.replace("\"", "")
        );
        server.get(&path).await.assert_status_unauthorized();

        let auth_user = db::create_user(&db, "auth_user".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &auth_user).await;
        test_util::test_access_control(
            &db,
            resource,
            resource_id,
            auth_user.user_id,
            AccessRole::Owner,
            async || server.get(&path).await,
        )
        .await;

        user_access_1.push(GetAccess {
            username: auth_user.username,
            access_role: AccessRole::Owner,
        });

        server
            .get("/api/Table/1000/access")
            .await
            .assert_status_not_found();

        let response = server.get(&path).await;
        response.assert_status_ok();
        let user_access_2: Vec<GetAccess> = response.json();
        test_util::assert_eq_vec(user_access_1, user_access_2, |a| a.username.to_owned());
        
        Ok(())
    }

    #[sqlx::test]
    async fn get_users_with_access(db: PgPool) -> anyhow::Result<()> {
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let user_1 = db::create_user(&db, "A".into(), "".into(), false).await?;
        let user_2 = db::create_user(&db, "B".into(), "".into(), false).await?;

        let resources = [
            (Resource::Table, table_id),
            (Resource::Dashboard, dashboard_id),
        ];
        for (idx, (resource, resource_id)) in resources.into_iter().enumerate() {
            db::create_access(
                &db,
                resource,
                resource_id,
                user_1.user_id,
                AccessRole::Owner,
            )
            .await?;
            db::create_access(
                &db,
                resource,
                resource_id,
                user_2.user_id,
                AccessRole::Viewer,
            )
            .await?;
            let user_ids = super::get_users_with_access(
                &db,
                1000,
                resource,
                resource_id,
                [user_1.username.clone(), user_2.username.clone()],
            )
            .await
            .unwrap();
            test_util::assert_eq_vec(user_ids, vec![user_1.user_id, user_2.user_id], |id| *id);

            assert!(matches!(
                super::get_users_with_access(
                    &db,
                    user_1.user_id,
                    resource,
                    resource_id,
                    [user_1.username.clone(), user_2.username.clone()],
                )
                .await,
                Err(ApiError::UnprocessableEntity(_))
            ));

            assert!(matches!(
                super::get_users_with_access(
                    &db,
                    1000,
                    resource,
                    resource_id,
                    [
                        user_1.username.clone(),
                        user_2.username.clone(),
                        format!("C{idx}")
                    ],
                )
                .await,
                Err(ApiError::UnprocessableEntity(_))
            ));
        }

        Ok(())
    }
}

//! Route handlers for managing users.
//!
//! User sessions are managed through an authentication cookie which is sent to the
//! front-end and received in the back-end to confirm identity.
//!

use crate::{
    AppState,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult, IntoAnyhow},
    model::users::{CreateUser, Credentials, SelectUser, UpdateUser, UserResponse},
};
use aide::{
    NoApi,
    axum::{
        ApiRouter,
        routing::{get_with, patch_with, post_with},
    },
};
use axum::{
    Form, Json,
    extract::{Path, State},
};
use axum_login::AuthSession;
use password_auth::generate_hash;
use tokio::task;

const INVALID_CREDENTIALS: &str = "Invalid credentials";
const ALREADY_LOGGED_IN: &str = "Already logged in";
const USERNAME_IS_TAKEN: &str = "Username is taken";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/login", post_with(login, docs::login))
        .api_route("/logout", get_with(logout, docs::logout))
        .api_route("/user", get_with(get_auth_user, docs::get_auth_user))
        .nest(
            "/users",
            ApiRouter::new()
                .api_route(
                    "/",
                    post_with(create_user, docs::create_user)
                        .get_with(get_all_users, docs::get_all_users),
                )
                .api_route(
                    "/{user_id}",
                    patch_with(update_user, docs::update_user)
                        .delete_with(delete_user, docs::delete_user),
                ),
        )
}

async fn login(
    mut session: AppAuthSession,
    Form(creds): Form<Credentials>,
) -> ApiResult<Json<UserResponse>> {
    if session.user.is_some() {
        return Err(ApiError::BadRequest(ALREADY_LOGGED_IN.into()));
    }
    let user = session
        .authenticate(creds)
        .await
        .anyhow()?
        .ok_or(ApiError::UnprocessableEntity(INVALID_CREDENTIALS.into()))?;

    session.login(&user).await.anyhow()?;

    Ok(Json(UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

pub async fn logout(mut session: AppAuthSession) -> ApiResult<()> {
    session.logout().await.anyhow()?;
    Ok(())
}

async fn get_auth_user(
    NoApi(AuthSession { user, .. }): AppAuthSession,
) -> ApiResult<Json<Option<UserResponse>>> {
    Ok(Json(user.map(|user| UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    })))
}

async fn create_user(
    State(AppState { db }): State<AppState>,
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    Form(create_user): Form<CreateUser>,
) -> ApiResult<Json<UserResponse>> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }

    let mut tx = db.begin().await?;

    if db::user_exists(tx.as_mut(), create_user.username.clone()).await? {
        return Err(ApiError::Conflict(USERNAME_IS_TAKEN.into()));
    }

    let password_hash = task::spawn_blocking(|| generate_hash(create_user.password))
        .await
        .anyhow()?;
    let user = db::create_user(tx.as_mut(), create_user.username, password_hash, false).await?;

    tx.commit().await?;
    Ok(Json(UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

async fn update_user(
    State(AppState { db }): State<AppState>,
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    Path(SelectUser { user_id }): Path<SelectUser>,
    Form(update_user): Form<UpdateUser>,
) -> ApiResult<Json<UserResponse>> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }

    let mut tx = db.begin().await?;

    if let Some(username) = update_user.username.clone()
        && db::user_exists(tx.as_mut(), username).await?
    {
        return Err(ApiError::Conflict(USERNAME_IS_TAKEN.into()));
    }

    let password_hash = if let Some(password) = update_user.password {
        Some(
            task::spawn_blocking(|| generate_hash(password))
                .await
                .anyhow()?,
        )
    } else {
        None
    };

    let user = db::update_user(
        tx.as_mut(),
        user_id,
        update_user.username,
        password_hash,
        None,
    )
    .await?;

    tx.commit().await?;
    Ok(Json(UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

async fn delete_user(
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    State(AppState { db }): State<AppState>,
    Path(SelectUser { user_id }): Path<SelectUser>,
) -> ApiResult<()> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }
    let mut tx = db.begin().await?;
    db::delete_user(tx.as_mut(), user_id).await?;
    tx.commit().await?;
    Ok(())
}

async fn get_all_users(
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    State(AppState { db }): State<AppState>,
) -> ApiResult<Json<Vec<UserResponse>>> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }
    let users = db::get_all_users(&db).await?;
    Ok(Json(users))
}

mod docs {
    use crate::{
        api::users::INVALID_CREDENTIALS,
        docs::{AUTHENTICATION_TAG, TransformOperationExt, USERS_TAG, template},
        model::users::UserResponse,
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    fn users<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, USERS_TAG)
            .response_description::<403, ()>("User is not an admin")
    }

    pub fn login(op: TransformOperation) -> TransformOperation {
        template::<Json<UserResponse>>(
            op,
            "login",
            "Login the user from the credentials.",
            false,
            AUTHENTICATION_TAG,
        )
        .response_description::<400, ()>("User is already authenticated")
        .response_description::<422, String>(INVALID_CREDENTIALS)
    }

    pub fn logout(op: TransformOperation) -> TransformOperation {
        template::<()>(
            op,
            "logout",
            "Logout the user. Does nothing if the user is not logged in.",
            false,
            AUTHENTICATION_TAG,
        )
    }

    pub fn get_auth_user(op: TransformOperation) -> TransformOperation {
        template::<Json<Option<UserResponse>>>(
            op,
            "get_auth_user",
            "Get the currently logged in user or nothing if not logged in.",
            false,
            AUTHENTICATION_TAG,
        )
    }

    pub fn create_user(op: TransformOperation) -> TransformOperation {
        users::<Json<UserResponse>>(
            op,
            "create_user",
            "Create a new user. Requires admin priviledges.",
        )
        .response_description::<409, ()>("Username is taken")
    }

    pub fn update_user(op: TransformOperation) -> TransformOperation {
        users::<Json<UserResponse>>(
            op,
            "update_user",
            "Update the user's username of password. Requires admin privileges.",
        )
        .response_description::<409, ()>("Username is taken")
    }

    pub fn delete_user(op: TransformOperation) -> TransformOperation {
        users::<()>(
            op,
            "delete_user",
            "Delete a user. Requires admin privileges.",
        )
    }

    pub fn get_all_users(op: TransformOperation) -> TransformOperation {
        users::<Json<Vec<UserResponse>>>(
            op,
            "get_all_users",
            "Retrieve all users. Requires admin privileges.",
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{
        AppConfig, Id,
        model::users::{Credentials, SelectUser, UserResponse},
        setup_tracing,
        test::{test_server, test_server_logged_in},
    };
    use password_auth::generate_hash;
    use serde_json::json;
    use sqlx::PgPool;
    use std::str::FromStr;
    use tower_sessions::{SessionStore, session};
    use tower_sessions_sqlx_store::PostgresStore;

    #[sqlx::test]
    async fn login(db: PgPool) -> anyhow::Result<()> {
        let server = test_server(db.clone()).await?;

        let credentials = Credentials {
            username: "john".into(),
            password: "1234".into(),
        };
        let user_id: Id = sqlx::query_scalar(
            r#"
                INSERT INTO app_user (username, password_hash)
                VALUES ($1, $2)
                RETURNING user_id
            "#,
        )
        .bind(&credentials.username)
        .bind(generate_hash(&credentials.password))
        .fetch_one(&db)
        .await?;
        let select_user = SelectUser { user_id };

        let response = server
            .post("/api/login")
            .form(&Credentials {
                username: credentials.username.clone(),
                password: "4321".into(),
            })
            .await;
        response.assert_status_unprocessable_entity();
        server.get("/test/user").await.assert_json(&None::<()>);

        let response = server
            .post("/api/login")
            .form(&credentials)
            .save_cookies()
            .await;
        response.assert_status_ok();
        response.assert_json(&UserResponse {
            user_id,
            username: credentials.username.clone(),
            is_admin: false,
        });
        server
            .get("/test/user")
            .await
            .assert_json_contains(&select_user);

        let response = server.post("/api/login").form(&credentials).await;
        response.assert_status_bad_request();
        server
            .get("/test/user")
            .await
            .assert_json_contains(&select_user);
        Ok(())
    }

    #[sqlx::test]
    async fn logout(db: PgPool) -> anyhow::Result<()> {
        let (server, _user_id) =
            test_server_logged_in(db.clone(), "molly", "1234", false).await?;

        let response = server.get("/api/logout").await;
        response.assert_status_ok();
        server.get("/test/user").await.assert_json(&None::<()>);

        let response = server.get("/api/logout").await;
        response.assert_status_ok();
        Ok(())
    }

    #[sqlx::test]
    async fn get_auth_user(db: PgPool) -> anyhow::Result<()> {
        setup_tracing();
        let username = "molly";
        let password = "1234";
        let is_admin = false;
        let (server, user_id) =
            test_server_logged_in(db.clone(), username, password, is_admin).await?;

        let response = server.get("/api/user").await;
        response.assert_status_ok();
        response.assert_json(&UserResponse {
            user_id,
            username: username.into(),
            is_admin,
        });

        let response = server.get("/api/user").clear_cookies().await;
        response.assert_status_ok();
        response.assert_json(&json!(null));

        Ok(())
    }
}

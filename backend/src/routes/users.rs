use anyhow::Context;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash,
};
use axum::{
    routing::{get, post},
    Extension, Json, Router,
};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult, OnConstraint},
    extractor::AuthUser,
    model::users::{LoginUser, NewUser, UpdateUserModel, User, UserBody},
    ApiContext,
};

use super::ApiState;

pub fn router() -> Router<ApiState> {
    // By having each module responsible for setting up its own routing,
    // it makes the root module a lot cleaner.
    Router::new()
        .route("/users", post(create_user))
        .route("/users/login", post(login_user))
        // .route("/user", get(get_current_user))
        .route("/user", get(get_current_user).put(update_user))
}

async fn create_user(
    ctx: Extension<ApiContext>,
    Json(req): Json<UserBody<NewUser>>,
) -> ApiResult<Json<UserBody<User>>> {
    let password_hash = hash_password(req.user.password).await?;

    let mut tx = ctx.db.begin().await?;

    let user: User = sqlx::query_as(
        r#"
            INSERT INTO "user" (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING
                user_id 
        "#,
    )
    .bind(&req.user.username)
    .bind(&req.user.email)
    .bind(password_hash)
    .fetch_one(tx.as_mut())
    .await
    .on_constraint("user_username_key", |_| {
        ApiError::unprocessable_entity([("username", "username taken")])
    })
    .on_constraint("user_email_key", |_| {
        ApiError::unprocessable_entity([("email", "email taken")])
    })?;

    let user_id = user.user_id;

    Ok(Json(UserBody {
        user: User {
            user_id,
            email: req.user.email,
            token: AuthUser { user_id }.to_jwt(&ctx),
            username: req.user.username,
            bio: "".to_string(),
            image: None,
        },
    }))
}

#[derive(FromRow)]
struct TempLoginUser {
    user_id: Uuid,
    email: String,
    token: String,
    username: String,
    bio: String,
    image: Option<String>,
    password_hash: String,
}
// https://realworld-docs.netlify.app/docs/specs/backend-specs/endpoints#authentication
async fn login_user(
    ctx: Extension<ApiContext>,
    Json(req): Json<UserBody<LoginUser>>,
) -> ApiResult<Json<UserBody<User>>> {
    let user: TempLoginUser = sqlx::query_as(
        r#"
            SELECT user_id, email, username, bio, image, password_hash 
            FROM "user" WHERE email = $1
        "#,
    )
    .bind(req.user.email)
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(ApiError::unprocessable_entity([(
        "email",
        "does not exist",
    )]))?;

    verify_password(req.user.password, user.password_hash).await?;

    Ok(Json(UserBody {
        user: User {
            user_id: user.user_id,
            email: user.email,
            token: AuthUser {
                user_id: user.user_id,
            }
            .to_jwt(&ctx),
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}

// https://realworld-docs.netlify.app/docs/specs/backend-specs/endpoints#get-current-user
async fn get_current_user(
    auth_user: AuthUser,
    ctx: Extension<ApiContext>,
) -> ApiResult<Json<UserBody<User>>> {
    let user: User =
        sqlx::query_as(r#"SELECT email, username, bio, image from "user" WHERE user_id = $1"#)
            .bind(auth_user.user_id)
            .fetch_one(&ctx.db)
            .await?;

    Ok(Json(UserBody {
        user: User {
            user_id: user.user_id,
            email: user.email,
            // The spec doesn't state whether we're supposed to return the same token we were passed,
            // or generate a new one. Generating a new one is easier the way the code is structured.
            //
            // This has the side-effect of automatically refreshing the session if the frontend
            // updates its token based on this response.
            token: auth_user.to_jwt(&ctx),
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}

// https://realworld-docs.netlify.app/docs/specs/backend-specs/endpoints#update-user
// Semantically, because this route allows a partial update it should be `PATCH`, not `PUT`.
// However, we have a spec to follow so `PUT` it is.
async fn update_user(
    auth_user: AuthUser,
    ctx: Extension<ApiContext>,
    Json(req): Json<UserBody<UpdateUserModel>>,
) -> ApiResult<Json<UserBody<User>>> {
    if req.user == UpdateUserModel::default() {
        // If there's no fields to update, these two routes are effectively identical.
        return get_current_user(auth_user, ctx).await;
    }

    // WTB `Option::map_async()`
    let password_hash = if let Some(password) = req.user.password {
        Some(hash_password(password).await?)
    } else {
        None
    };

    let user: User = sqlx::query_as(
        // This is how we do optional updates of fields without needing a separate query for each.
        // language=PostgreSQL
        r#"
            update "user"
            set email = coalesce($1, "user".email),
                username = coalesce($2, "user".username),
                password_hash = coalesce($3, "user".password_hash),
                bio = coalesce($4, "user".bio),
                image = coalesce($5, "user".image)
            where user_id = $6
            returning email, username, bio, image
        "#,
    )
    .bind(req.user.email)
    .bind(req.user.username)
    .bind(password_hash)
    .bind(req.user.bio)
    .bind(req.user.image)
    .bind(auth_user.user_id)
    .fetch_one(&ctx.db)
    .await
    .on_constraint("user_username_key", |_| {
        ApiError::unprocessable_entity([("username", "username taken")])
    })
    .on_constraint("user_email_key", |_| {
        ApiError::unprocessable_entity([("email", "email taken")])
    })?;

    Ok(Json(UserBody {
        user: User {
            user_id: user.user_id,
            email: user.email,
            token: auth_user.to_jwt(&ctx),
            username: user.username,
            bio: user.bio,
            image: user.image,
        },
    }))
}

async fn hash_password(password: String) -> ApiResult<String> {
    // Argon2 hashing is designed to be computationally intensive,
    // so we need to do this on a blocking thread.
    Ok(tokio::task::spawn_blocking(move || -> ApiResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(PasswordHash::generate(Argon2::default(), password, &salt)
            .map_err(|e| anyhow::anyhow!("failed to generate password hash: {}", e))?
            .to_string())
    })
    .await
    .context("panic in generating password hash")??)
}

async fn verify_password(password: String, password_hash: String) -> ApiResult<()> {
    Ok(tokio::task::spawn_blocking(move || -> ApiResult<()> {
        let hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow::anyhow!("invalid password hash: {}", e))?;

        hash.verify_password(&[&Argon2::default()], password)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => ApiError::Unauthorized,
                _ => anyhow::anyhow!("failed to verify password hash: {}", e).into(),
            })
    })
    .await
    .context("panic in verifying password hash")??)
}

use crate::{
    error::{ApiError, OnConstraint},
    extractor::AuthUser,
    model::users::{LoginUser, NewUser, User, UserBody, UserIdentifier},
    ApiContext, Id,
};
use axum::{Extension, Json};
use sqlx::{Acquire, PgExecutor, Postgres, Result};

// https://realworld-docs.netlify.app/docs/specs/backend-specs/endpoints#registration
// pub async fn create_user(
//     conn: impl Acquire<'_, Database = Postgres>,
//     Json(req): Json<UserBody<NewUser>>,
// ) -> Result<Json<UserBody<User>>> {
//     let password_hash = hash_password(req.user.password).await?;
//
//     let mut tx = conn.begin().await?;
//
//     // let user_id = sqlx::query_scalar!(
//     //     // language=PostgreSQL
//     //     r#"insert into "user" (username, email, password_hash) values ($1, $2, $3) returning user_id"#,
//     //     req.user.username,
//     //     req.user.email,
//     //     password_hash
//     // )
//     // .fetch_one(tx.as_mut())
//     // .await
//     // .on_constraint("user_username_key", |_| {
//     //     ApiError::unprocessable_entity([("username", "username taken")])
//     // })
//     // .on_constraint("user_email_key", |_| {
//     //     ApiError::unprocessable_entity([("email", "email taken")])
//     // })?;
//     let user: User = sqlx::query_as(
//         r#"
//             INSERT INTO "user" (username, email, password_hash)
//             VALUES ($1, $2, $3)
//             RETURNING
//                 user_id,
//                 email,
//                 token,
//                 username,
//                 bio
//                 image,
//         "#,
//     )
//     .bind(req.user.username)
//     .bind(req.user.email)
//     .bind(password_hash)
//     .fetch_one(tx.as_mut())
//     .await
//     .on_constraint("user_username_key", |_| {
//         ApiError::unprocessable_entity([("username", "username taken")])
//     })
//     .on_constraint("user_email_key", |_| {
//         ApiError::unprocessable_entity([("email", "email taken")])
//     })?;
//
//
//     let user_id = user.user_id;
//     // let user_ident = UserIdentifier::new(user_id, "data_view");
//     // // A view which always returns zero rows
//     // sqlx::query(&format!(
//     //     r#"
//     //         CREATE VIEW {user_ident} AS
//     //         SELECT NULL WHERE FALSE
//     //     "#
//     // ))
//     // .execute(tx.as_mut())
//     // .await?;
//     //
//     // tx.commit().await?;
//
//
//     Ok(Json(UserBody {
//         user: User {
//             user_id,
//             email: req.user.email,
//             token: AuthUser { user_id }.to_jwt(&ctx),
//             username: req.user.username,
//             bio: "".to_string(),
//             image: None,
//         },
//     }))
// }

// https://realworld-docs.netlify.app/docs/specs/backend-specs/endpoints#authentication
// async fn login_user(
//     ctx: Extension<ApiContext>,
//     Json(req): Json<UserBody<LoginUser>>,
// ) -> Result<Json<UserBody<User>>> {
//     let user = sqlx::query!(
//         r#"
//             select user_id, email, username, bio, image, password_hash
//             from "user" where email = $1
//         "#,
//         req.user.email,
//     )
//     .fetch_optional(&ctx.db)
//     .await?
//     .ok_or(Error::unprocessable_entity([("email", "does not exist")]))?;
//
//     verify_password(req.user.password, user.password_hash).await?;
//
//     Ok(Json(UserBody {
//         user: User {
//             user_id:
//             email: user.email,
//             token: AuthUser {
//                 user_id: user.user_id,
//             }
//             .to_jwt(&ctx),
//             username: user.username,
//             bio: user.bio,
//             image: user.image,
//         },
//     }))
// }
//
// // https://realworld-docs.netlify.app/docs/specs/backend-specs/endpoints#get-current-user
// async fn get_current_user(
//     auth_user: AuthUser,
//     ctx: Extension<ApiContext>,
// ) -> Result<Json<UserBody<User>>> {
//     let user = sqlx::query!(
//         r#"select email, username, bio, image from "user" where user_id = $1"#,
//         auth_user.user_id
//     )
//     .fetch_one(&ctx.db)
//     .await?;
//
//     Ok(Json(UserBody {
//         user: User {
//             email: user.email,
//             // The spec doesn't state whether we're supposed to return the same token we were passed,
//             // or generate a new one. Generating a new one is easier the way the code is structured.
//             //
//             // This has the side-effect of automatically refreshing the session if the frontend
//             // updates its token based on this response.
//             token: auth_user.to_jwt(&ctx),
//             username: user.username,
//             bio: user.bio,
//             image: user.image,
//         },
//     }))
// }
//
// // https://realworld-docs.netlify.app/docs/specs/backend-specs/endpoints#update-user
// // Semantically, because this route allows a partial update it should be `PATCH`, not `PUT`.
// // However, we have a spec to follow so `PUT` it is.
// async fn update_user(
//     auth_user: AuthUser,
//     ctx: Extension<ApiContext>,
//     Json(req): Json<UserBody<UpdateUser>>,
// ) -> Result<Json<UserBody<User>>> {
//     if req.user == UpdateUser::default() {
//         // If there's no fields to update, these two routes are effectively identical.
//         return get_current_user(auth_user, ctx).await;
//     }
//
//     // WTB `Option::map_async()`
//     let password_hash = if let Some(password) = req.user.password {
//         Some(hash_password(password).await?)
//     } else {
//         None
//     };
//
//     let user = sqlx::query!(
//         // This is how we do optional updates of fields without needing a separate query for each.
//         // language=PostgreSQL
//         r#"
//             update "user"
//             set email = coalesce($1, "user".email),
//                 username = coalesce($2, "user".username),
//                 password_hash = coalesce($3, "user".password_hash),
//                 bio = coalesce($4, "user".bio),
//                 image = coalesce($5, "user".image)
//             where user_id = $6
//             returning email, username, bio, image
//         "#,
//         req.user.email,
//         req.user.username,
//         password_hash,
//         req.user.bio,
//         req.user.image,
//         auth_user.user_id
//     )
//     .fetch_one(&ctx.db)
//     .await
//     .on_constraint("user_username_key", |_| {
//         Error::unprocessable_entity([("username", "username taken")])
//     })
//     .on_constraint("user_email_key", |_| {
//         Error::unprocessable_entity([("email", "email taken")])
//     })?;
//
//     Ok(Json(UserBody {
//         user: User {
//             email: user.email,
//             token: auth_user.to_jwt(&ctx),
//             username: user.username,
//             bio: user.bio,
//             image: user.image,
//         },
//     }))
// }
//
// async fn hash_password(password: String) -> Result<String> {
//     // Argon2 hashing is designed to be computationally intensive,
//     // so we need to do this on a blocking thread.
//     Ok(tokio::task::spawn_blocking(move || -> Result<String> {
//         let salt = SaltString::generate(rand::thread_rng());
//         Ok(
//             PasswordHash::generate(Argon2::default(), password, salt.as_str())
//                 .map_err(|e| anyhow::anyhow!("failed to generate password hash: {}", e))?
//                 .to_string(),
//         )
//     })
//     .await
//     .context("panic in generating password hash")??)
// }
//
// async fn verify_password(password: String, password_hash: String) -> Result<()> {
//     Ok(tokio::task::spawn_blocking(move || -> Result<()> {
//         let hash = PasswordHash::new(&password_hash)
//             .map_err(|e| anyhow::anyhow!("invalid password hash: {}", e))?;
//
//         hash.verify_password(&[&Argon2::default()], password)
//             .map_err(|e| match e {
//                 argon2::password_hash::Error::Password => Error::Unauthorized,
//                 _ => anyhow::anyhow!("failed to verify password hash: {}", e).into(),
//             })
//     })
//     .await
//     .context("panic in verifying password hash")??)
// }

// TESTING
pub async fn debug_get_user_id(executor: impl PgExecutor<'_>) -> sqlx::Result<Id> {
    Ok(
        sqlx::query_as::<_, (_,)>("SELECT user_id FROM app_user LIMIT 1;")
            .fetch_one(executor)
            .await?
            .0,
    )
}

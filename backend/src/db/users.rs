use crate::{
    Id, db,
    model::users::{User, UserResponse},
};
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

pub async fn create_user(
    conn: impl Acquire<'_, Database = Postgres>,
    username: String,
    password_hash: String,
    is_admin: bool,
) -> sqlx::Result<User> {
    let mut tx = conn.begin().await?;
    let user = sqlx::query_as(
        r#"
            INSERT INTO app_user (username, password_hash, is_admin)
            VALUES ($1, $2, $3)
            RETURNING *
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .bind(is_admin)
    .fetch_one(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(user)
}

pub async fn update_user(
    conn: impl Acquire<'_, Database = Postgres>,
    user_id: Id,
    username: Option<String>,
    password_hash: Option<String>,
    is_admin: Option<bool>,
) -> sqlx::Result<User> {
    let mut tx: sqlx::Transaction<'_, _> = conn.begin().await?;
    let mut query = QueryBuilder::new(
        r#"
        UPDATE app_user SET
    "#,
    );
    let mut comma = false;
    let mut check_comma = |query: &mut QueryBuilder<'_, Postgres>| {
        if comma {
            query.push(" , ");
        }
        comma = true;
    };
    if let Some(username) = username {
        check_comma(&mut query);
        query.push(" username = ").push_bind(username);
    }
    if let Some(password_hash) = password_hash {
        check_comma(&mut query);
        query.push(" password_hash = ").push_bind(password_hash);
    }
    if let Some(is_admin) = is_admin {
        check_comma(&mut query);
        query.push(" is_admin = ").push_bind(is_admin);
    }
    let user: User = query
        .push(r#" WHERE user_id = "#)
        .push_bind(user_id)
        .push(r#" RETURNING *"#)
        .build_query_as()
        .fetch_one(tx.as_mut())
        .await?;
    tx.commit().await?;
    Ok(user)
}

pub async fn delete_user(
    conn: impl Acquire<'_, Database = Postgres>,
    user_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    sqlx::query(
        r#"
        DELETE FROM app_user
        WHERE user_id = $1
    "#,
    )
    .bind(user_id)
    .execute(tx.as_mut())
    .await?;
    db::delete_tables_without_owner(tx.as_mut()).await?;
    db::delete_dashboards_without_owner(tx.as_mut()).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_all_users(executor: impl PgExecutor<'_>) -> sqlx::Result<Vec<UserResponse>> {
    sqlx::query_as(
        r#"
            SELECT
                user_id,
                username,
                is_admin
            FROM app_user
        "#,
    )
    .fetch_all(executor)
    .await
}

pub async fn get_user_by_id(
    executor: impl PgExecutor<'_>,
    user_id: Id,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as(
        r#"
            SELECT
                user_id,
                username,
                password_hash,
                is_admin
            FROM app_user
            WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(executor)
    .await
}

pub async fn get_user_by_username(
    executor: impl PgExecutor<'_>,
    username: String,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as(
        r#"
            SELECT
                user_id,
                username,
                password_hash,
                is_admin
            FROM app_user
            WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(executor)
    .await
}

pub async fn user_exists_by_username(
    executor: impl PgExecutor<'_>,
    username: String,
) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM app_user
                WHERE username = $1
            )
        "#,
    )
    .bind(username)
    .fetch_one(executor)
    .await
}

pub async fn user_exists_by_id(executor: impl PgExecutor<'_>, user_id: Id) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM app_user
                WHERE user_id = $1
            )
        "#,
    )
    .bind(user_id)
    .fetch_one(executor)
    .await
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        model::users::{User, UserResponse},
        test_util,
    };
    use sqlx::PgPool;

    #[sqlx::test]
    async fn create_user(db: PgPool) -> anyhow::Result<()> {
        let username = "test";
        let password_hash = "password";
        let is_admin = false;
        let user_1 =
            super::create_user(&db, username.into(), password_hash.into(), is_admin).await?;
        assert_eq!(username, user_1.username);
        assert_eq!(password_hash, user_1.password_hash);
        assert_eq!(is_admin, user_1.is_admin);
        let user_2: User = sqlx::query_as(r#"SELECT * FROM app_user WHERE user_id = $1"#)
            .bind(user_1.user_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(user_1, user_2);
        Ok(())
    }

    #[sqlx::test]
    async fn update_user(db: PgPool) -> anyhow::Result<()> {
        let user = super::create_user(&db, "john".into(), "1234".into(), false).await?;
        let username = "jane";
        let password_hash = "5678";
        let is_admin = true;
        let user_1 = super::update_user(
            &db,
            user.user_id,
            Some(username.into()),
            Some(password_hash.into()),
            Some(is_admin),
        )
        .await?;
        assert_eq!(username, user_1.username);
        assert_eq!(password_hash, user_1.password_hash);
        assert_eq!(is_admin, user_1.is_admin);
        let user_2: User = sqlx::query_as(r#"SELECT * FROM app_user WHERE user_id = $1"#)
            .bind(user_1.user_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(user_1, user_2);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_user(db: PgPool) -> anyhow::Result<()> {
        let user = super::create_user(&db, "john".into(), "1234".into(), false).await?;
        super::delete_user(&db, user.user_id).await?;
        let not_exists: bool =
            sqlx::query_scalar(r#"SELECT NOT EXISTS (SELECT 1 FROM app_user WHERE user_id = $1)"#)
                .bind(user.user_id)
                .fetch_one(&db)
                .await?;
        assert!(not_exists);
        Ok(())
    }

    #[sqlx::test]
    async fn get_all_users(db: PgPool) -> anyhow::Result<()> {
        let mut users_1: Vec<UserResponse> = Vec::new();
        for (username, is_admin) in [("python", false), ("kotlin", false), ("typescript", true)] {
            let user = super::create_user(&db, username.into(), "1234".into(), is_admin).await?;
            users_1.push(UserResponse {
                user_id: user.user_id,
                username: user.username,
                is_admin: user.is_admin,
            });
        }
        let users_2 = super::get_all_users(&db).await?;
        test_util::assert_eq_vec(users_1, users_2, |u| u.user_id);
        Ok(())
    }

    #[sqlx::test]
    async fn get_user_by_id(db: PgPool) -> anyhow::Result<()> {
        assert!(super::get_user_by_id(&db, 0).await?.is_none());
        let user_1 = super::create_user(&db, "john".into(), "1234".into(), false).await?;
        let user_2 = super::get_user_by_id(&db, user_1.user_id).await?.unwrap();
        assert_eq!(user_1, user_2);
        Ok(())
    }

    #[sqlx::test]
    async fn get_user_from_username(db: PgPool) -> anyhow::Result<()> {
        assert!(
            super::get_user_by_username(&db, "john".into())
                .await?
                .is_none()
        );
        let user_1 = super::create_user(&db, "john".into(), "1234".into(), false).await?;
        let user_2 = super::get_user_by_username(&db, user_1.username.clone())
            .await?
            .unwrap();
        assert_eq!(user_1, user_2);
        Ok(())
    }

    #[sqlx::test]
    async fn user_exists_by_id(db: PgPool) -> anyhow::Result<()> {
        assert!(!super::user_exists_by_id(&db, 1).await?);
        let user = super::create_user(&db, "john".into(), "1234".into(), false).await?;
        assert!(super::user_exists_by_id(&db, user.user_id).await?);
        Ok(())
    }

    #[sqlx::test]
    async fn user_exists_by_username(db: PgPool) -> anyhow::Result<()> {
        assert!(!super::user_exists_by_username(&db, "john".into()).await?);
        let user = super::create_user(&db, "john".into(), "1234".into(), false).await?;
        assert!(super::user_exists_by_username(&db, user.username).await?);
        Ok(())
    }
}

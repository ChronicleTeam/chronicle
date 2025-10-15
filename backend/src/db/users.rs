use crate::{
    Id,
    model::users::{AccessRole, User, UserResponse},
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

pub async fn get_user(executor: impl PgExecutor<'_>, user_id: Id) -> sqlx::Result<Option<User>> {
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

pub async fn get_user_from_username(
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

pub async fn user_exists(executor: impl PgExecutor<'_>, username: String) -> sqlx::Result<bool> {
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

pub async fn create_access(
    conn: impl Acquire<'_, Database = Postgres>,
    users: impl IntoIterator<Item = (Id, AccessRole)>,
    resource_id: Id,
    table_name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    QueryBuilder::new(format!(
        r#"INSERT INTO {table_name} (user_id, resource_id, access_role)"#
    ))
    .push_values(users, |mut builder, (user_id, access_role)| {
        builder
            .push_bind(user_id)
            .push_bind(resource_id)
            .push_bind(access_role);
    })
    .build()
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn update_access(
    conn: impl Acquire<'_, Database = Postgres>,
    users: impl IntoIterator<Item = (Id, AccessRole)>,
    resource_id: Id,
    table_name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    QueryBuilder::new(format!(
        r#"
            UPDATE {table_name} AS t
            SET access_role = v.access_role
            FROM (
        "#
    ))
    .push_values(users, |mut builder, (user_id, access_role)| {
        builder.push_bind(user_id).push_bind(access_role);
    })
    .push(format!(
        r#"
            ) AS v(user_id, access_role)
            WHERE t.user_id = v.user_id
            AND t.resource_id = 
        "#
    ))
    .push_bind(resource_id)
    .build()
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn delete_access(
    conn: impl Acquire<'_, Database = Postgres>,
    users: impl IntoIterator<Item = Id>,
    resource_id: Id,
    table_name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    QueryBuilder::new(format!(r#"DELETE FROM {table_name} WHERE resource_id = "#))
        .push_bind(resource_id)
        .push(format!(" AND user_id IN ("))
        .push_values(users, |mut builder, user_id| {
            builder.push_bind(user_id);
        })
        .push(")")
        .build()
        .execute(tx.as_mut())
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Return the [Relation] between the user and this table.
pub async fn get_access(
    executor: impl PgExecutor<'_>,
    user_id: Id,
    resource_id: Id,
    table_name: &str,
) -> sqlx::Result<Option<AccessRole>> {
    sqlx::query_scalar::<_, AccessRole>(&format!(
        r#"
            SELECT access_role
            FROM {table_name}
            WHERE user_id = $1 AND resource_id = $2
        "#
    ))
    .bind(user_id)
    .bind(resource_id)
    .fetch_optional(executor)
    .await
}

#[cfg(test)]
mod test {
    use crate::{
        Id, db,
        model::users::{User, UserResponse},
    };
    use password_auth::{generate_hash, verify_password};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn create_user(db: PgPool) -> anyhow::Result<()> {
        let username = "test";
        let password = "password";
        let is_admin = false;
        let user1 =
            db::create_user(&db, username.into(), generate_hash(password), is_admin).await?;
        assert_eq!(username, user1.username);
        assert_eq!(is_admin, user1.is_admin);
        verify_password(password, &user1.password_hash)?;
        let user2: User = sqlx::query_as(
            r#"
                SELECT *
                FROM app_user
                WHERE user_id = $1
            "#,
        )
        .bind(user1.user_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(user1, user2);
        Ok(())
    }

    #[sqlx::test]
    async fn update_user(db: PgPool) -> anyhow::Result<()> {
        let user_id: Id = sqlx::query_scalar(
            r#"
                INSERT INTO app_user (username, password_hash, is_admin)
                VALUES ($1, $2, $3)
                RETURNING user_id
            "#,
        )
        .bind("john")
        .bind("1234")
        .bind(false)
        .fetch_one(&db)
        .await?;
        let username = "jane";
        let password_hash = "5678";
        let is_admin = true;
        let user1 = db::update_user(
            &db,
            user_id,
            Some(username.into()),
            Some(password_hash.into()),
            Some(is_admin),
        )
        .await?;
        assert_eq!(username, user1.username);
        assert_eq!(password_hash, user1.password_hash);
        assert_eq!(is_admin, user1.is_admin);
        let user2: User = sqlx::query_as(
            r#"
                SELECT *
                FROM app_user
                WHERE user_id = $1
            "#,
        )
        .bind(user1.user_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(user1, user2);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_user(db: PgPool) -> anyhow::Result<()> {
        let user_id: Id = sqlx::query_scalar(
            r#"
                INSERT INTO app_user (username, password_hash)
                VALUES ($1, $2)
                RETURNING user_id
            "#,
        )
        .bind("john")
        .bind("1234")
        .fetch_one(&db)
        .await?;
        db::delete_user(&db, user_id).await?;
        let not_exists: bool = sqlx::query_scalar(
            r#"
                SELECT NOT EXISTS (
                    SELECT 1
                    FROM app_user
                    WHERE user_id = $1
                )
            "#,
        )
        .bind(user_id)
        .fetch_one(&db)
        .await?;
        assert!(not_exists);
        Ok(())
    }

    #[sqlx::test]
    async fn get_all_users(db: PgPool) -> anyhow::Result<()> {
        let mut users1: Vec<UserResponse> = Vec::new();
        for username in ["python", "kotlin", "typescript"] {
            users1.push(
                sqlx::query_as(
                    r#"
                    INSERT INTO app_user (username, password_hash)
                    VALUES ($1, $2)
                    RETURNING *
                "#,
                )
                .bind(username)
                .bind("1234")
                .fetch_one(&db)
                .await?,
            );
        }
        let mut users2 = db::get_all_users(&db).await?;
        users1.sort_by_key(|u| u.user_id);
        users2.sort_by_key(|u| u.user_id);
        assert!(users1.len() == users2.len());
        assert!(
            users1
                .into_iter()
                .zip(users2)
                .all(|(user1, user2)| user1 == user2)
        );
        Ok(())
    }

    #[sqlx::test]
    async fn get_user(db: PgPool) -> anyhow::Result<()> {
        assert!(db::get_user(&db, 0).await?.is_none());
        let user1: User = sqlx::query_as(
            r#"
                INSERT INTO app_user (username, password_hash)
                VALUES ($1, $2)
                RETURNING *
            "#,
        )
        .bind("john")
        .bind("1234")
        .fetch_one(&db)
        .await?;
        let user2 = db::get_user(&db, user1.user_id).await?.unwrap();
        assert_eq!(user1, user2);
        Ok(())
    }

    #[sqlx::test]
    async fn get_user_from_username(db: PgPool) -> anyhow::Result<()> {
        assert!(
            db::get_user_from_username(&db, "john".into())
                .await?
                .is_none()
        );
        let user1: User = sqlx::query_as(
            r#"
                INSERT INTO app_user (username, password_hash)
                VALUES ($1, $2)
                RETURNING *
            "#,
        )
        .bind("john")
        .bind("1234")
        .fetch_one(&db)
        .await?;
        let user2 = db::get_user_from_username(&db, user1.username.clone())
            .await?
            .unwrap();
        assert_eq!(user1, user2);
        Ok(())
    }

    #[sqlx::test]
    async fn user_exists(db: PgPool) -> anyhow::Result<()> {
        assert!(!db::user_exists(&db, "john".into()).await?);
        let username: String = sqlx::query_scalar(
            r#"
                INSERT INTO app_user (username, password_hash)
                VALUES ($1, $2)
                RETURNING username
            "#,
        )
        .bind("john")
        .bind("1234")
        .fetch_one(&db)
        .await?;
        assert!(db::user_exists(&db, username).await?);
        Ok(())
    }

    // #[sqlx::test]
    // async fn create_access(db: PgPool) -> anyhow::Result<()> {
    //     Ok(())
    // }

    // #[sqlx::test]
    // async fn update_access(db: PgPool) -> anyhow::Result<()> {
    //     Ok(())
    // }

    // #[sqlx::test]
    // async fn delete_access(db: PgPool) -> anyhow::Result<()> {
    //     Ok(())
    // }
}

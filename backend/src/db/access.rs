use crate::{
    Id,
    model::access::{AccessRole, GetAccess, Resource},
};
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

pub async fn create_access(
    conn: impl Acquire<'_, Database = Postgres>,
    resource: Resource,
    resource_id: Id,
    user_id: Id,
    access_role: AccessRole,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let tablename = resource.access_tablename();
    sqlx::query(&format!(
        r#"
        INSERT INTO {tablename} (user_id, resource_id, access_role)
        VALUES ($1, $2, $3)
    "#
    ))
    .bind(user_id)
    .bind(resource_id)
    .bind(access_role)
    .execute(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn update_many_access(
    conn: impl Acquire<'_, Database = Postgres>,
    resource: Resource,
    resource_id: Id,
    user_access_roles: impl IntoIterator<Item = (Id, AccessRole)>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let tablename = resource.access_tablename();
    QueryBuilder::new(format!(
        r#"
            UPDATE {tablename} AS t
            SET access_role = v.access_role
            FROM (
        "#
    ))
    .push_values(user_access_roles, |mut builder, (user_id, access_role)| {
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

pub async fn delete_many_access(
    conn: impl Acquire<'_, Database = Postgres>,
    resource: Resource,
    resource_id: Id,
    user_ids: impl IntoIterator<Item = Id>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let tablename = resource.access_tablename();
    QueryBuilder::new(format!(r#"DELETE FROM {tablename} WHERE resource_id = "#))
        .push_bind(resource_id)
        .push(format!(" AND user_id IN ("))
        .push_values(user_ids, |mut builder, user_id| {
            builder.push_bind(user_id);
        })
        .push(")")
        .build()
        .execute(tx.as_mut())
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_all_access(
    executor: impl PgExecutor<'_>,
    resource: Resource,
    resource_id: Id,
) -> sqlx::Result<Vec<GetAccess>> {
    let tablename = resource.access_tablename();
    sqlx::query_as(&format!(
        r#"
            SELECT username, access_role
            FROM {tablename} AS a
            JOIN app_user AS u
            ON a.user_id = u.user_id
            WHERE resource_id = $1

        "#
    ))
    .bind(resource_id)
    .fetch_all(executor)
    .await
}

pub async fn get_access_role(
    executor: impl PgExecutor<'_>,
    resource: Resource,
    resource_id: Id,
    user_id: Id,
) -> sqlx::Result<Option<AccessRole>> {
    let tablename = resource.access_tablename();
    sqlx::query_scalar::<_, AccessRole>(&format!(
        r#"
            SELECT access_role
            FROM {tablename}
            WHERE user_id = $1 AND resource_id = $2
        "#
    ))
    .bind(user_id)
    .bind(resource_id)
    .fetch_optional(executor)
    .await
}

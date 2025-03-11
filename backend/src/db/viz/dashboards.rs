use sqlx::{Acquire, PgExecutor, Postgres};

use crate::{
    db::Relation,
    model::viz::{CreateDashboard, Dashboard, UpdateDashboard},
    Id,
};

pub async fn create_dashboard(
    executor: impl PgExecutor<'_>,
    user_id: Id,
    CreateDashboard { name, description }: CreateDashboard,
) -> sqlx::Result<Dashboard> {
    sqlx::query_as(
        r#"
            INSERT INTO dashboard (user_id, name, description)
            VALUES ($1, $2, $3) 
            RETURNING
                dashboard_id,
                user_id,
                name,
                description,
                created_at,
                updated_at
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(description)
    .fetch_one(executor)
    .await
}

pub async fn update_dashboard(
    executor: impl PgExecutor<'_>,
    dashboard_id: Id,
    UpdateDashboard { name, description }: UpdateDashboard,
) -> sqlx::Result<Dashboard> {
    sqlx::query_as(
        r#"
            UPDATE dashboard
            SET name = $1, description = $2
            WHERE dashboard_id = $3
            RETURNING
                dashboard_id,
                user_id,
                name,
                description,
                created_at,
                updated_at
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(dashboard_id)
    .fetch_one(executor)
    .await
}

pub async fn delete_dashboard(
    conn: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    let data_view_names: Vec<String> = sqlx::query_scalar(
        r#"
            DELETE FROM chart
            WHERE dashboard_id = $1
            RETURNING data_view_name
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(tx.as_mut())
    .await?;

    for data_view_name in data_view_names {
        _ = sqlx::query(&format!(r#"DROP VIEW {data_view_name}"#))
            .execute(tx.as_mut())
            .await?;
    }

    _ = sqlx::query(
        r#"
            DELETE FROM dashboard
            WHERE dashboard_id = $1
        "#,
    )
    .bind(dashboard_id)
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_dashboards(
    executor: impl PgExecutor<'_>,
    user_id: Id,
) -> sqlx::Result<Vec<Dashboard>> {
    sqlx::query_as(
        r#"
            SELECT
                dashboard_id,
                user_id,
                name,
                description,
                created_at,
                updated_at
            FROM dashboard
            WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
}

pub async fn check_dashboard_relation(
    executor: impl PgExecutor<'_>,
    user_id: Id,
    dashboard_id: Id,
) -> sqlx::Result<Relation> {
    sqlx::query_scalar::<_, Id>(
        r#"
            SELECT user_id
            FROM dashboard
            WHERE dashboard_id = $1
        "#,
    )
    .bind(dashboard_id)
    .fetch_optional(executor)
    .await
    .map(|id| match id {
        None => Relation::Absent,
        Some(id) if id == user_id => Relation::Owned,
        Some(_) => Relation::NotOwned,
    })
}

use crate::{
    db, model::{
        access::AccessRole, viz::{CreateDashboard, Dashboard, GetDashboard, UpdateDashboard}
    }, Id
};
use sqlx::{Acquire, PgExecutor, Postgres};

/// Add a dashboard to this user.
pub async fn create_dashboard(
    conn: impl Acquire<'_, Database = Postgres>,
    CreateDashboard { name, description }: CreateDashboard,
) -> sqlx::Result<Dashboard> {
    let mut tx = conn.begin().await?;

    let dashboard: Dashboard = sqlx::query_as(
        r#"
            INSERT INTO dashboard (name, description)
            VALUES ($1, $2) 
            RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(dashboard)
}

/// Update the dashboard metadata.
pub async fn update_dashboard(
    conn: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
    UpdateDashboard { name, description }: UpdateDashboard,
) -> sqlx::Result<Dashboard> {
    let mut tx = conn.begin().await?;

    let dashboard: Dashboard = sqlx::query_as(
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
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(dashboard)
}

/// Delete the dashboard along with its charts.
pub async fn delete_dashboard(
    conn: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    let chart_ids: Vec<Id> = sqlx::query_scalar(
        r#"
            SELECT chart_id
            FROM chart
            WHERE dashboard_id = $1
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(tx.as_mut())
    .await?;
    for chart_id in chart_ids {
        db::delete_chart(tx.as_mut(), chart_id).await?;
    }

    sqlx::query(
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

/// Get all dashboards belonging to this user.
pub async fn get_dashboards(
    executor: impl PgExecutor<'_>,
    user_id: Id,
) -> sqlx::Result<Vec<GetDashboard>> {
    sqlx::query_as(
        r#"
            SELECT *
            FROM dashboard AS d
            JOIN dashboard_access AS a
            ON d.dashboard_id = a.resource_id
            WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
}

pub async fn delete_dashboards_without_owner(
    conn: impl Acquire<'_, Database = Postgres>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let dashboard_ids: Vec<Id> = sqlx::query_scalar(
        r#"
        SELECT dashboard_id
        FROM dashboard AS d
        WHERE NOT EXISTS (
            SELECT 1
            FROM dashboard_access AS a
            WHERE a.resource_id = d.dashboard_id
            AND a.access_role = $1
        )
    "#,
    )
    .bind(AccessRole::Owner)
    .fetch_all(tx.as_mut())
    .await?;

    for dashboard_id in dashboard_ids {
        delete_dashboard(tx.as_mut(), dashboard_id).await?;
    }
    tx.commit().await?;
    Ok(())
}

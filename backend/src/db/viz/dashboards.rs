use crate::{
    db::Relation,
    model::viz::{ChartIdentifier, CreateDashboard, Dashboard, UpdateDashboard},
    Id,
};
use sqlx::{Acquire, PgExecutor, Postgres};

/// Add a dashboard to this user.
pub async fn create_dashboard(
    conn: impl Acquire<'_, Database = Postgres>,
    user_id: Id,
    CreateDashboard { name, description }: CreateDashboard,
) -> sqlx::Result<Dashboard> {
    let mut tx = conn.begin().await?;

    let dashboard: Dashboard = sqlx::query_as(
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
            DELETE FROM chart
            WHERE dashboard_id = $1
            RETURNING chart_id
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(tx.as_mut())
    .await?;

    for chart_id in chart_ids {
        let chart_ident = ChartIdentifier::new(chart_id, "data_view");
        sqlx::query(&format!(r#"DROP VIEW {chart_ident}"#))
            .execute(tx.as_mut())
            .await?;
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

/// Return the [Relation] between the user and this dashboard.
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

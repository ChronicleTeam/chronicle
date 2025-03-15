use std::collections::HashMap;

use crate::{
    db::Relation,
    model::{viz::{AxisField, AxisIdentifier, Chart, ChartData, ChartIdentifier, CreateChart, UpdateChart}, Cell},
    Id,
};
use sqlx::{Acquire, PgExecutor, Postgres};

pub async fn create_chart(
    conn: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
    CreateChart {
        table_id,
        title,
        chart_kind,
    }: CreateChart,
) -> sqlx::Result<Chart> {
    let mut tx = conn.begin().await?;

    let chart: Chart = sqlx::query_as(
        r#"
            INSERT INTO chart (dashboard_id, table_id, title, chart_kind)
            VALUES ($1, $2, $3, $4)
            RETURNING
                chart_id,
                dashboard_id,
                table_id,
                title,
                chart_kind,
                created_at,
                updated_at
        "#,
    )
    .bind(dashboard_id)
    .bind(table_id)
    .bind(title)
    .bind(chart_kind)
    .fetch_one(tx.as_mut())
    .await?;


    let chart_ident = ChartIdentifier::new(chart.chart_id, "data_view");
    // A view which always returns zero rows
    sqlx::query(&format!(
        r#"
            CREATE VIEW {chart_ident} AS
            SELECT NULL WHERE FALSE
        "#
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(chart)
}

pub async fn update_chart(
    conn: impl Acquire<'_, Database = Postgres>,
    chart_id: Id,
    UpdateChart { title, chart_kind }: UpdateChart,
) -> sqlx::Result<Chart> {
    let mut tx = conn.begin().await?;

    let chart = sqlx::query_as(
        r#"
            UPDATE chart
            SET title = $1, chart_kind = $2
            WHERE chart_id = $3
            RETURNING
                chart_id,
                dashboard_id,
                table_id,
                title,
                chart_kind,
                created_at,
                updated_at
        "#,
    )
    .bind(title)
    .bind(chart_kind)
    .bind(chart_id)
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(chart)
}

pub async fn delete_chart(
    conn: impl Acquire<'_, Database = Postgres>,
    chart_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(
        r#"
            DELETE FROM chart
            WHERE chart_id = $1
        "#,
    )
    .bind(chart_id)
    .execute(tx.as_mut())
    .await?;

    let chart_ident = ChartIdentifier::new(chart_id, "data_view");
    sqlx::query(&format!(r#"DROP VIEW {chart_ident}"#))
        .execute(tx.as_mut())
        .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_chart_table_id(executor: impl PgExecutor<'_>, chart_id: Id) -> sqlx::Result<Id> {
    sqlx::query_scalar(
        r#"
        SELECT table_id
        FROM chart
        WHERE chart_id = $1
    "#,
    )
    .bind(chart_id)
    .fetch_one(executor)
    .await
}

pub async fn get_charts(
    executor: impl PgExecutor<'_> + Copy,
    dashboard_id: Id,
) -> sqlx::Result<Vec<Chart>> {
    sqlx::query_as(
        r#"
            SELECT
                chart_id,
                dashboard_id,
                table_id,
                title,
                chart_kind,
                created_at,
                updated_at
            FROM chart
            WHERE dashboard_id = $1
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(executor)
    .await
}



pub async fn get_chart_data(
    executor: impl PgExecutor<'_> + Copy,
    chart_id: Id,
) -> sqlx::Result<ChartData> {
    let chart: Chart = sqlx::query_as(
        r#"
            SELECT
                chart_id,
                dashboard_id,
                table_id,
                title,
                chart_kind,
                created_at,
                updated_at
            FROM chart
            WHERE chart_id = $1
        "#,
    )
    .bind(chart_id)
    .fetch_one(executor)
    .await?;

    let axes: Vec<AxisField> = sqlx::query_as(
        r#"
            SELECT
                a.axis_id,
                a.chart_id,
                a.field_id,
                a.axis_kind,
                a.aggregate,
                a.created_at,
                a.updated_at,
                f.name AS field_name,
                f.field_kind
            FROM axis AS a
            JOIN meta_field AS f
            ON a.field_id = f.field_id
            WHERE a.chart_id = $1
        "#,
    )
    .bind(chart_id)
    .fetch_all(executor)
    .await?;

    let chart_ident = ChartIdentifier::new(chart_id, "data_view");
    let rows = sqlx::query(&format!(r#"SELECT * FROM {chart_ident}"#))
        .fetch_all(executor)
        .await?;

    let mut cells: Vec<HashMap<Id, Cell>> = Vec::new();

    for row in rows {
        let mut entry = HashMap::new();
        for AxisField {
            axis, field_kind, ..
        } in &axes
        {
            let axis_ident = AxisIdentifier::new(axis.axis_id);
            entry.insert(
                axis.axis_id,
                axis.aggregate.as_ref().map_or_else(
                    || Cell::from_field_row(&row, &axis_ident.unquoted(), &field_kind),
                    |aggregate| {
                        Cell::from_aggregate_row(
                            &row,
                            &axis_ident.unquoted(),
                            aggregate,
                            &field_kind,
                        )
                    },
                )?,
            );
        }
        cells.push(entry);
    }

    Ok(ChartData { chart, axes, cells })
}


pub async fn check_chart_relation(
    executor: impl PgExecutor<'_>,
    dashboard_id: Id,
    chart_id: Id,
) -> sqlx::Result<Relation> {
    sqlx::query_scalar::<_, Id>(
        r#"
            SELECT dashboard_id
            FROM chart
            WHERE chart_id = $1
        "#,
    )
    .bind(chart_id)
    .fetch_optional(executor)
    .await
    .map(|id| match id {
        None => Relation::Absent,
        Some(id) if id == dashboard_id => Relation::Owned,
        Some(_) => Relation::NotOwned,
    })
}
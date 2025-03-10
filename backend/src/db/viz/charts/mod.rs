mod axes;

pub use axes::*;

use crate::{
    db::Relation,
    model::{
        viz::{AxisData, Chart, CreateChart},
        Cell, CellMap,
    },
    Id,
};
use sqlx::PgExecutor;
use std::collections::HashMap;

pub async fn create_chart(
    executor: impl PgExecutor<'_>,
    dashboard_id: Id,
    CreateChart {
        title,
        chart_kind,
    }: CreateChart,
) -> sqlx::Result<Chart> {
    sqlx::query_as(
        r#"
            INSERT INTO chart (dashboard_id, title, chart_kind)
            VALUES ($1, $2, $3)
            RETURNING
                chart_id,
                dashboard_id,
                title,
                chart_kind,
                data_view_name,
                created_at,
                updated_at
        "#,
    )
    .bind(dashboard_id)
    .bind(title)
    .bind(chart_kind)
    .fetch_one(executor)
    .await
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

async fn get_data_view(
    executor: impl PgExecutor<'_>,
    data_view_name: &str,
    axis_data: &HashMap<Id, AxisData>,
) -> sqlx::Result<Vec<CellMap>> {
    let sql = &format!(r#"SELECT * FROM {data_view_name}"#);

    let rows = sqlx::query(sql).fetch_all(executor).await?;

    let mut cells: Vec<CellMap> = Vec::new();

    for row in rows {
        let mut cell_entry = CellMap::new();
        for AxisData { axis, field } in axis_data.values() {
            cell_entry.insert(
                axis.axis_id,
                axis.aggregate.as_ref().map_or_else(
                    || Cell::from_field_row(&row, &axis.data_item_name, &field.field_kind),
                    |aggregate| {
                        Cell::from_aggregate_row(
                            &row,
                            &axis.data_item_name,
                            aggregate,
                            &field.field_kind,
                        )
                    },
                )?,
            );
        }
        cells.push(cell_entry);
    }

    Ok(cells)
}

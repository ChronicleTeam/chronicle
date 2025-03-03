use std::collections::HashMap;

use itertools::Itertools;
use sqlx::{Acquire, Postgres};

use crate::{
    model::viz::{Axis, ChartData, ChartKind, MarkKind},
    Id,
};

pub async fn create_chart(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    title: String,
    chart_kind: ChartKind,
    x_axis: Axis,
    y_axis: Axis,
    marks: HashMap<MarkKind, Axis>,
) -> sqlx::Result<ChartData> {
    let mut tx = connection.begin().await?;

    let chart_id: i32 = sqlx::query_scalar(
        r#"
            WITH new_chart AS (
                INSERT INTO chart (dashboard_id, title, chart_kind, x_axis, y_axis)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING chart_id
            )
            INSERT INTO mark (chart_id, field_id, mark_kind)
            SELECT new_chart.chart_id, unnest($6::int[]), unnest($7::mark_kind[]) FROM new_chart
            RETURNING new_chart.chart_id
        "#,
    )
    .bind(table_id)
    .bind(title)
    .bind(chart_kind)
    .bind(x_axis)
    .bind(y_axis)
    .bind(marks.keys().collect_vec())
    .bind(marks.values().collect_vec())
    .fetch_one(tx.as_mut())
    .await?;

    todo!()
}

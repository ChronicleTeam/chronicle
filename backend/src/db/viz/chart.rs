use std::collections::HashMap;

use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

use crate::{
    model::{
        data::{Field, FullField},
        viz::{Aggregate, Axis, Chart, ChartData, ChartKind, Mark, MarkKind},
    },
    Id,
};

pub async fn create_chart(
    connection: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
    title: String,
    chart_kind: ChartKind,
    x_axis: Axis,
    y_axis: Axis,
    marks: HashMap<MarkKind, Axis>,
) -> sqlx::Result<ChartData> {
    let mut tx = connection.begin().await?;

    let chart: Chart = sqlx::query_as(
        r#"
            INSERT INTO chart (dashboard_id, title, chart_kind, x_axis, y_axis)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                chart_id,
                dashboard_id,
                title,
                chart_kind,
                x_axis,
                y_axis,
                created_at,
                updated_at
        "#,
    )
    .bind(dashboard_id)
    .bind(title)
    .bind(chart_kind)
    .bind(x_axis)
    .bind(y_axis)
    .fetch_one(tx.as_mut())
    .await?;

    let marks: HashMap<MarkKind, Axis> = sqlx::query_as(
        r#"
            INSERT INTO mark (chart_id, field_id, mark_kind)
            SELECT $1, unnest($2::mark_kind[]), unnest($3::axis[])
            RETURNING mark_kind, axis
        "#,
    )
    .bind(chart.chart_id)
    .bind(marks.keys().collect_vec())
    .bind(marks.values().collect_vec())
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .collect();

    let x_field: Field = sqlx::query_as(
        r#"
            SELECT
                field_id,
                table_id,
                name,
                field_kind,
                created_at,
                updated_at
            FROM meta_field
            WHERE field_id = $1
        "#,
    )
    .bind(chart.x_axis.field_id)
    .fetch_one(tx.as_mut())
    .await?;

    let chart_data_query = format!(
        r#"

    "#
    );

    todo!()
}

async fn get_field_axis(
    executor: impl PgExecutor<'_>,
    field_id: Id,
    aggregate: Option<Aggregate>,
) -> (Field, String) {
    let FullField {
        field,
        data_field_name,
    } = sqlx::query_as(
        r#"
            SELECT
                field_id,
                table_id,
                name,
                field_kind,
                created_at,
                updated_at,
                data_field_name
            FROM meta_field
            WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_one(&executor)
    .await?;

    let mut identifier = data_field_name;

    if let Some(aggregate) = aggregate {
        let identifier = format!(
            "{}({data_field_name})",
            match aggregate {
                Aggregate::Sum => "SUM",
                Aggregate::Average => "AVG",
                Aggregate::Count => "COUNT",
            }
        );
    }

    (field, todo!())
}

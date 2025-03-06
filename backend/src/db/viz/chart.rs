use std::collections::HashMap;

use crate::{
    model::{
        data::{Cell, CellEntry, Field},
        viz::{Aggregate, Axis, AxisData, Chart, ChartData, CreateChart},
    },
    Id,
};
use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

pub async fn create_chart(
    connection: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
    CreateChart {
        table_id,
        title,
        chart_kind,
        axes,
    }: CreateChart,
) -> sqlx::Result<ChartData> {
    let mut tx = connection.begin().await?;

    let chart: Chart = sqlx::query_as(
        r#"
            INSERT INTO chart (dashboard_id, table_id, title, chart_kind)
            VALUES ($1. $2, $3, $4)
            RETURNING
                chart_id,
                dashboard_id,
                table_id,
                title,
                chart_kind,
                data_view_name,
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

    let mut axes: Vec<Axis> =
        QueryBuilder::new(r#"INSERT INTO axis (chart_id, field_id, axis_kind, aggregate)"#)
            .push_values(axes, |mut b, axis| {
                b.push_bind(chart.chart_id)
                    .push_bind(axis.field_id)
                    .push_bind(axis.axis_kind)
                    .push_bind(axis.aggregate);
            })
            .push(
                r#"
                    RETURNING
                        axis_id,
                        chart_id,
                        field_id,
                        axis_kind,
                        aggregate,
                        data_item_name,
                        created_at,
                        updated_at
                "#,
            )
            .build_query_as()
            .fetch_all(tx.as_mut())
            .await?;

    let mut fields: Vec<Field> = sqlx::query_as(
        r#"
            SELECT
                field_id,
                table_id,
                name,
                field_kind,
                data_field_name,
                created_at,
                updated_at
            FROM axis AS a
            JOIN meta_field AS f
            ON a.field_id = f.field_id
            WHERE chart_id = $1
        "#,
    )
    .bind(chart.chart_id)
    .fetch_all(tx.as_mut())
    .await?;

    axes.sort_by_key(|field| field.field_id);
    fields.sort_by_key(|field| field.field_id);

    let axis_data_map: HashMap<_, _> = axes
        .into_iter()
        .zip(fields)
        .map(|(axis, field)| (axis.axis_id, AxisData { axis, field }))
        .collect();

    let select_columns = axis_data_map
        .iter()
        .map(|(_, AxisData { axis, field })| {
            let identifier = if let Some(aggregate) = &axis.aggregate {
                &format!(
                    "{}({})",
                    match aggregate {
                        Aggregate::Sum => "SUM",
                        Aggregate::Average => "AVG",
                        Aggregate::Count => "COUNT",
                    },
                    field.data_field_name
                )
            } else {
                &field.data_field_name
            };
            format!("{identifier} AS {}", axis.data_item_name)
        })
        .join(", ");

    let data_table_name: String = sqlx::query_scalar(
        r#"
            SELECT data_table_name
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(tx.as_mut())
    .await?;

    let data_view_name = &chart.data_view_name;

    sqlx::query(&format!(
        r#"
            CREATE {data_view_name} AS
            SELECT {select_columns}
            FROM {data_table_name}
        "#
    ))
    .execute(tx.as_mut())
    .await?;

    let cells = get_data_view(tx.as_mut(), &data_view_name, &axis_data_map).await?;

    tx.commit().await?;

    Ok(ChartData {
        chart,
        axis_data_map,
        cells,
    })
}

async fn get_data_view(
    executor: impl PgExecutor<'_>,
    data_view_name: &str,
    axis_data: &HashMap<Id, AxisData>,
) -> sqlx::Result<Vec<CellEntry>> {
    let sql = &format!(r#"SELECT * FROM {data_view_name}"#);

    let rows = sqlx::query(sql).fetch_all(executor).await?;

    let mut cells: Vec<CellEntry> = Vec::new();

    for row in rows {
        let mut cell_entry = CellEntry::new();
        for AxisData { axis, field } in axis_data.values() {
            cell_entry.insert(
                axis.axis_id,
                Cell::from_row(&row, &axis.data_item_name, &field.field_kind)?,
            );
        }
        cells.push(cell_entry);
    }

    Ok(cells)
}

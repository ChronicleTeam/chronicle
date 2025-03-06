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
    connection: impl Acquire<'_, Database = Postgres> + Clone,
    dashboard_id: Id,
    CreateChart {
        table_id,
        title,
        chart_kind,
        axes,
    }: CreateChart,
) -> sqlx::Result<ChartData> {
    let mut tx = connection.clone().begin().await?;

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
                f.field_id,
                f.table_id,
                f.name,
                f.field_kind,
                f.data_field_name,
                f.created_at,
                f.updated_at
            FROM axis AS a
            JOIN meta_field AS f
            ON a.field_id = f.field_id
            WHERE chart_id = $1
        "#,
    )
    .bind(chart.chart_id)
    .fetch_all(tx.as_mut())
    .await?;

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

    axes.sort_by_key(|field| field.field_id);
    fields.sort_by_key(|field| field.field_id);

    let axis_data_map: HashMap<_, _> = axes
        .into_iter()
        .zip(fields)
        .map(|(axis, field)| (axis.axis_id, AxisData { axis, field }))
        .collect();

    let mut group_by_columns = Vec::new();
    let mut select_columns = Vec::new();
    for AxisData { axis, field } in axis_data_map.values() {
        let identifier = if let Some(aggregate) = &axis.aggregate {
            &format!(
                "{}({})::{}",
                match aggregate {
                    Aggregate::Sum => "SUM",
                    Aggregate::Average => "AVG",
                    Aggregate::Min => "MIN",
                    Aggregate::Max => "MAX",
                    Aggregate::Count => "COUNT",
                },
                field.data_field_name,
                field.field_kind.get_sql_type(),
            )
        } else {
            group_by_columns.push(field.data_field_name.clone());
            &field.data_field_name
        };
        select_columns.push(format!("{identifier} AS {}", axis.data_item_name));
    }
    let group_by_columns = group_by_columns.join(", ");
    let select_columns = select_columns.join(", ");

    let group_by_statement = if group_by_columns.len() > 0 {
        format!("GROUP BY {group_by_columns}")
    } else {
        String::new()
    };

    let data_view_name = &chart.data_view_name;

    println!(
        r#"
            CREATE VIEW {data_view_name} AS
            SELECT {select_columns}
            FROM {data_table_name}
            {group_by_statement}
        "#
    );

    sqlx::query(&format!(
        r#"
            CREATE VIEW {data_view_name} AS
            SELECT {select_columns}
            FROM {data_table_name}
            {group_by_statement}
        "#
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    let mut tx = connection.begin().await?;

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

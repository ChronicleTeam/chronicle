use crate::{
    model::{
        data::{Field, FullField},
        viz::{
            Aggregate, Axis, Chart, ChartData, ChartKind, CreateChart, FullChart, Mark, MarkKind,
        },
    },
    Id,
};
use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres};

pub async fn create_chart(
    connection: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
    CreateChart {
        table_id,
        title,
        chart_kind,
        axes
    }: CreateChart,
) -> sqlx::Result<ChartData> {
    let mut tx = connection.begin().await?;

    

    let FullChart {
        chart,
        data_view_name,
    } = sqlx::query_as(
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

    let (mark_kinds, mark_axes): (Vec<_>, Vec<_>) = marks.into_iter().unzip();

    let marks: Vec<Mark> = sqlx::query_as(
        r#"
            INSERT INTO mark (chart_id, field_id, mark_kind)
            SELECT $1, unnest($2::mark_kind[]), unnest($3::axis[])
            RETURNING mark_id, chart_id, mark_kind, axis
        "#,
    )
    .bind(chart.chart_id)
    .bind(mark_kinds)
    .bind(mark_axes.clone())
    .fetch_all(tx.as_mut())
    .await?;

    let (x_field, x_identifier) =
        get_field_axis(tx.as_mut(), chart.x_axis.field_id, chart.x_axis.aggregate).await?;
    let (y_field, y_identifier) =
        get_field_axis(tx.as_mut(), chart.y_axis.field_id, chart.y_axis.aggregate).await?;

    let (mut mark_fields, mut mark_identifiers) = (Vec::new(), Vec::new());
    for Mark { axis, .. } in &marks {
        let (field, identifier) =
            get_field_axis(tx.as_mut(), axis.field_id, axis.aggregate.clone()).await?;
        mark_fields.push(field);
        mark_identifiers.push(identifier);
    }

    let select_columns = [
        format!("{x_identifier} AS x"),
        format!("{y_identifier} AS y"),
    ]
    .into_iter()
    .chain(
        mark_identifiers
            .into_iter()
            .zip(marks)
            .map(|(ident, Mark { mark_id, .. })| format!("{ident} AS _{mark_id}")),
    )
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

    assert!([x_field.table_id, y_field.table_id]
        .into_iter()
        .chain(mark_fields.iter().map(|f| f.table_id))
        .all(|id| id == table_id));

    sqlx::query(&format!(
        r#"
            CREATE {data_view_name} AS
            SELECT {select_columns}
            FROM {data_table_name}
        "#
    ))
    .execute(tx.as_mut())
    .await?;

    todo!()
}

async fn get_field_axis(
    executor: impl PgExecutor<'_>,
    field_id: Id,
    aggregate: Option<Aggregate>,
) -> sqlx::Result<(Field, String)> {
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
    .fetch_one(executor)
    .await?;

    let identifier = if let Some(aggregate) = aggregate {
        format!(
            "{}({data_field_name})",
            match aggregate {
                Aggregate::Sum => "SUM",
                Aggregate::Average => "AVG",
                Aggregate::Count => "COUNT",
            }
        )
    } else {
        data_field_name
    };

    Ok((field, identifier))
}

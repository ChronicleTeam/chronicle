use crate::model::{
    data::{Field, Table},
    viz::{Axis, Chart, CreateAxis},
};
use sqlx::{Acquire, Postgres, QueryBuilder};

pub async fn set_axes(
    connection: impl Acquire<'_, Database = Postgres> + Clone,
    chart: Chart,
    table: Table,
    axes_and_fields: Vec<(CreateAxis, Field)>,
) -> sqlx::Result<Vec<Axis>> {
    let mut tx = connection.clone().begin().await?;

    let (axes, fields): (Vec<_>, Vec<_>) = axes_and_fields.into_iter().unzip();

    _ = sqlx::query(
        r#"
            DELETE FROM axis
            WHERE chart_id = $1
        "#,
    )
    .bind(chart.chart_id)
    .execute(tx.as_mut())
    .await?;

    let axes: Vec<Axis> =
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

    let data_table_name = table.data_table_name;

    let mut group_by_columns = Vec::new();
    let mut select_columns = Vec::new();
    for (axis, field) in axes.iter().zip(fields) {
        let identifier = if let Some(aggregate) = &axis.aggregate {
            &format!(
                "{}({})::{}",
                aggregate.get_sql_aggregate(),
                field.data_field_name,
                aggregate.get_sql_type(&field.field_kind.0),
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

    Ok(axes)
}

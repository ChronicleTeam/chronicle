use crate::{
    Id,
    model::{
        data::{FieldIdentifier, FieldKind, TableIdentifier},
        viz::{Axis, AxisIdentifier, ChartIdentifier, CreateAxis},
    },
};
use sqlx::{Acquire, Postgres, QueryBuilder, types::Json};

/// Set the list of axes of this chart using the given table as data source.
pub async fn set_axes(
    conn: impl Acquire<'_, Database = Postgres>,
    chart_id: Id,
    table_id: Id,
    axes: Vec<CreateAxis>,
) -> sqlx::Result<Vec<Axis>> {
    let mut tx = conn.begin().await?;

    sqlx::query(
        r#"
            DELETE FROM axis
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

    if axes.is_empty() {
        sqlx::query(&format!(
            r#"
                CREATE VIEW {chart_ident} AS
                SELECT NULL WHERE FALSE
            "#
        ))
        .execute(tx.as_mut())
        .await?;
        return Ok(Vec::new());
    }

    let axes: Vec<Axis> =
        QueryBuilder::new(r#"INSERT INTO axis (chart_id, field_id, axis_kind, aggregate)"#)
            .push_values(axes, |mut builder, axis| {
                builder
                    .push_bind(chart_id)
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
                        created_at,
                        updated_at
                "#,
            )
            .build_query_as()
            .fetch_all(tx.as_mut())
            .await?;

    let mut group_by_columns = Vec::new();
    let mut select_columns = Vec::new();
    for axis in &axes {
        let field_ident = FieldIdentifier::new(axis.field_id);
        let item = if let Some(aggregate) = &axis.aggregate {
            let Json(field_kind): Json<FieldKind> = sqlx::query_scalar(
                r#"
                    SELECT field_kind
                    FROM meta_field
                    WHERE field_id = $1
                "#,
            )
            .bind(axis.field_id)
            .fetch_one(tx.as_mut())
            .await?;
            &format!(
                "{}({})::{}",
                aggregate.get_sql_aggregate(),
                field_ident,
                aggregate.get_sql_type(&field_kind),
            )
        } else {
            group_by_columns.push(field_ident.to_string());
            &field_ident.to_string()
        };
        let axis_ident = AxisIdentifier::new(axis.axis_id);
        select_columns.push(format!("{item} AS {}", axis_ident));
    }
    let group_by_columns = group_by_columns.join(", ");
    let select_columns = select_columns.join(", ");

    let group_by_statement = if group_by_columns.len() > 0 {
        format!("GROUP BY {group_by_columns}")
    } else {
        String::new()
    };

    let table_ident = TableIdentifier::new(table_id, "data_table");
    sqlx::query(&format!(
        r#"
            CREATE VIEW {chart_ident} AS
            SELECT {select_columns}
            FROM {table_ident}
            {group_by_statement}
        "#
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(axes)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use sqlx::PgPool;
    use crate::{db, model::{data::CreateTable, viz::{ChartKind, CreateChart, CreateDashboard}}};

    #[sqlx::test]
    async fn set_axes(db: PgPool) -> anyhow::Result<()> {
        // Things to test:
        // axis SQL table
        // dynamic view data
        // all aggregates + 1 group by column
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let create_chart = CreateChart {
            table_id,
            name: "test".into(),
            chart_kind: ChartKind::Bar,
        };
        let chart_id = db::create_chart(&db, dashboard_id, create_chart.clone()).await?.chart_id;
        

        Ok(())
    }
}
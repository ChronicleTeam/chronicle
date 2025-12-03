use std::collections::HashMap;

use crate::{
    Id,
    model::{
        Cell,
        viz::{
            AxisField, AxisIdentifier, Chart, ChartData, ChartIdentifier, CreateChart, UpdateChart,
        },
    },
};
use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres};

/// Add a chart to this dashboard and create the actual SQL view.
pub async fn create_chart(
    conn: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
    CreateChart {
        table_id,
        name,
        chart_kind,
    }: CreateChart,
) -> sqlx::Result<Chart> {
    let mut tx = conn.begin().await?;

    let chart: Chart = sqlx::query_as(
        r#"
            INSERT INTO chart (dashboard_id, table_id, name, chart_kind)
            VALUES ($1, $2, $3, $4)
            RETURNING
                chart_id,
                dashboard_id,
                table_id,
                name,
                chart_kind,
                created_at,
                updated_at
        "#,
    )
    .bind(dashboard_id)
    .bind(table_id)
    .bind(name)
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

/// Update the chart metadata.
pub async fn update_chart(
    conn: impl Acquire<'_, Database = Postgres>,
    chart_id: Id,
    UpdateChart { name, chart_kind }: UpdateChart,
) -> sqlx::Result<Chart> {
    let mut tx = conn.begin().await?;

    let chart = sqlx::query_as(
        r#"
            UPDATE chart
            SET name = $1, chart_kind = $2
            WHERE chart_id = $3
            RETURNING
                chart_id,
                dashboard_id,
                table_id,
                name,
                chart_kind,
                created_at,
                updated_at
        "#,
    )
    .bind(name)
    .bind(chart_kind)
    .bind(chart_id)
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(chart)
}

/// Delete this chart along with the actual SQL view and the axes.
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

/// Delete this chart along with the actual SQL view and the axes.
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

/// Get all the charts of this dashboard.
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
                name,
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

/// Get the chart, its axes and associated fields, and its data points.
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
                name,
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
    let select_columns = axes
        .iter()
        .map(|axis_field| AxisIdentifier::new(axis_field.axis.axis_id))
        .join(", ");
    let rows = sqlx::query(&format!(
        r#"
            SELECT {select_columns}
            FROM {chart_ident}
        "#
    ))
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
                    || Cell::from_field_row(&row, &axis_ident.unquoted(), field_kind),
                    |aggregate| {
                        Cell::from_aggregate_row(
                            &row,
                            &axis_ident.unquoted(),
                            aggregate,
                            field_kind,
                        )
                    },
                )?,
            );
        }
        cells.push(entry);
    }

    Ok(ChartData { chart, axes, cells })
}

pub async fn chart_exists(
    executor: impl PgExecutor<'_>,
    dashboard_id: Id,
    chart_id: Id,
) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM chart
                WHERE dashboard_id = $1 AND chart_id = $2
            )
        "#,
    )
    .bind(dashboard_id)
    .bind(chart_id)
    .fetch_one(executor)
    .await
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        db,
        model::{
            Cell,
            data::{CreateField, CreateTable, FieldKind, FieldMetadata},
            viz::{
                Aggregate, AxisField, AxisKind, ChartIdentifier, ChartKind, CreateAxis,
                CreateChart, CreateDashboard, UpdateChart,
            },
        },
        test_util,
    };
    use sqlx::PgPool;
    use std::collections::HashMap;

    #[sqlx::test]
    async fn create_chart(db: PgPool) -> anyhow::Result<()> {
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
        let chart_1 = super::create_chart(&db, dashboard_id, create_chart.clone()).await?;
        assert_eq!(create_chart.table_id, chart_1.table_id);
        assert_eq!(create_chart.name, chart_1.name);
        assert_eq!(create_chart.chart_kind, chart_1.chart_kind);
        let chart_2 = sqlx::query_as(r#"SELECT * FROM chart WHERE chart_id = $1"#)
            .bind(chart_1.chart_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(chart_1, chart_2);
        let chart_ident = ChartIdentifier::new(chart_1.chart_id, "data_view");
        sqlx::query(&format!(r#"SELECT FROM {chart_ident}"#))
            .execute(&db)
            .await
            .unwrap();
        Ok(())
    }

    #[sqlx::test]
    async fn update_chart(db: PgPool) -> anyhow::Result<()> {
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
        let chart_id = super::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id,
                name: "test".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?
        .chart_id;
        let update_chart = UpdateChart {
            name: "X Over Time".into(),
            chart_kind: ChartKind::Line,
        };
        let chart_1 = super::update_chart(&db, chart_id, update_chart.clone()).await?;
        let chart_2 = sqlx::query_as(r#"SELECT * FROM chart WHERE chart_id = $1"#)
            .bind(chart_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(chart_1, chart_2);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_chart(db: PgPool) -> anyhow::Result<()> {
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
        let field_id = db::create_field(
            &db,
            table_id,
            CreateField {
                name: "X".into(),
                field_kind: FieldKind::Checkbox,
            },
        )
        .await?
        .field_id;
        let chart_id = super::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id,
                name: "test".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?
        .chart_id;
        let axis_id = db::set_axes(
            &db,
            chart_id,
            table_id,
            vec![CreateAxis {
                field_id,
                axis_kind: AxisKind::X,
                aggregate: None,
            }],
        )
        .await?
        .first()
        .unwrap()
        .axis_id;
        super::delete_chart(&db, chart_id).await?;
        let not_exists: bool =
            sqlx::query_scalar(r#"SELECT NOT EXISTS (SELECT 1 FROM chart WHERE chart_id = $1) AND NOT EXISTS (SELECT 1 FROM axis WHERE axis_id = $2)"#)
                .bind(chart_id)
                .bind(axis_id)
                .fetch_one(&db)
                .await?;
        assert!(not_exists);
        let chart_ident = ChartIdentifier::new(chart_id, "data_view");
        sqlx::query(&format!(r#"SELECT FROM {chart_ident}"#))
            .execute(&db)
            .await
            .unwrap_err();
        Ok(())
    }

    #[sqlx::test]
    async fn get_chart_table_id(db: PgPool) -> anyhow::Result<()> {
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let table_id_1 = db::create_table(
            &db,
            CreateTable {
                name: "test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let chart_id = super::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id: table_id_1,
                name: "test".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?
        .chart_id;
        let table_id_2 = super::get_chart_table_id(&db, chart_id).await?;
        assert_eq!(table_id_1, table_id_2);
        Ok(())
    }

    #[sqlx::test]
    async fn get_charts(db: PgPool) -> anyhow::Result<()> {
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
        let mut charts_1 = Vec::new();
        for (idx, chart_kind) in [ChartKind::Bar, ChartKind::Line, ChartKind::Table]
            .into_iter()
            .enumerate()
        {
            charts_1.push(
                super::create_chart(
                    &db,
                    dashboard_id,
                    CreateChart {
                        table_id,
                        name: idx.to_string(),
                        chart_kind,
                    },
                )
                .await?,
            );
        }
        let charts_2 = super::get_charts(&db, dashboard_id).await?;
        test_util::assert_eq_vec(charts_1, charts_2, |c| c.chart_id);
        Ok(())
    }

    #[sqlx::test]
    async fn get_chart_data(db: PgPool) -> anyhow::Result<()> {
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
        let checkbox_field = db::create_field(
            &db,
            table_id,
            CreateField {
                name: "Task Complete".into(),
                field_kind: FieldKind::Checkbox,
            },
        )
        .await?;
        let checkbox_id = checkbox_field.field_id;
        let integer_field = db::create_field(
            &db,
            table_id,
            CreateField {
                name: "Time (hours)".into(),
                field_kind: FieldKind::Integer {
                    is_required: false,
                    range_start: None,
                    range_end: None,
                },
            },
        )
        .await?;
        let integer_id = integer_field.field_id;

        let _entries = db::create_entries(
            &db,
            table_id,
            None,
            vec![
                FieldMetadata::from_field(checkbox_field.clone()),
                FieldMetadata::from_field(integer_field.clone()),
            ],
            vec![
                vec![Cell::Boolean(false), Cell::Integer(2)],
                vec![Cell::Boolean(true), Cell::Integer(1)],
                vec![Cell::Boolean(true), Cell::Integer(3)],
                vec![Cell::Boolean(false), Cell::Integer(4)],
                vec![Cell::Boolean(true), Cell::Integer(5)],
            ],
        )
        .await?;

        let chart = super::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id,
                name: "test".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?;
        let mut axes = db::set_axes(
            &db,
            chart.chart_id,
            table_id,
            vec![
                CreateAxis {
                    field_id: checkbox_field.field_id,
                    axis_kind: AxisKind::X,
                    aggregate: None,
                },
                CreateAxis {
                    field_id: integer_field.field_id,
                    axis_kind: AxisKind::Y,
                    aggregate: Some(Aggregate::Sum),
                },
            ],
        )
        .await?;
        axes.sort_by_key(|a| a.field_id);

        let chart_data = super::get_chart_data(&db, chart.chart_id).await?;
        assert_eq!(chart, chart_data.chart);

        let mut fields = vec![checkbox_field, integer_field];
        fields.sort_by_key(|f| f.field_id);
        let axes_fields = axes
            .into_iter()
            .zip(fields)
            .map(|(axis, field)| AxisField {
                axis,
                field_name: field.name,
                field_kind: field.field_kind,
            })
            .collect();
        test_util::assert_eq_vec(axes_fields, chart_data.axes, |af| af.axis.axis_id);

        let cells = vec![
            HashMap::from([
                (checkbox_id, Cell::Boolean(false)),
                (integer_id, Cell::Decimal(6.into())),
            ]),
            HashMap::from([
                (checkbox_id, Cell::Boolean(true)),
                (integer_id, Cell::Decimal(9.into())),
            ]),
        ];
        test_util::assert_eq_vec(cells, chart_data.cells, |row| {
            let Cell::Boolean(v) = row[&checkbox_id] else {
                panic!()
            };
            v
        });
        Ok(())
    }

    #[sqlx::test]
    async fn chart_exists(db: PgPool) -> anyhow::Result<()> {
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let table_id_1 = db::create_table(
            &db,
            CreateTable {
                name: "test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        assert!(!super::chart_exists(&db, dashboard_id, 1).await?);
        let chart_id = super::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id: table_id_1,
                name: "test".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?
        .chart_id;
        assert!(super::chart_exists(&db, dashboard_id, chart_id).await?);
        Ok(())
    }
}

//! Database functions for managing chart axes.

use crate::{
    Id,
    model::{
        data::{FieldIdentifier, FieldKind, TableIdentifier},
        viz::{Axis, AxisIdentifier, ChartIdentifier, CreateAxis},
    },
};
use sqlx::{Acquire, Postgres, QueryBuilder, types::Json};

/// Set the axes of this chart using the given table as data source and replace the SQL view.
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
        select_columns.push(format!("{item} AS {axis_ident}"));
    }
    let group_by_columns = group_by_columns.join(", ");
    let select_columns = select_columns.join(", ");

    let group_by_statement = if !group_by_columns.is_empty() {
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
    use crate::{
        db,
        model::{
            Cell,
            data::{CreateField, CreateTable, FieldKind, FieldMetadata},
            viz::{Aggregate, AxisKind, ChartKind, CreateAxis, CreateChart, CreateDashboard},
        },
    };
    use chrono::DateTime;
    use itertools::Itertools;
    use sqlx::PgPool;
    use std::collections::{HashMap, HashSet};

    const TIMESTAMP: i64 = 1761696082;
    const ROW_COUNT: usize = 10;

    fn member_row() -> Vec<Cell> {
        [
            "Chris", "Chris", "Chris", "Jane", "Jane", "Jane", "Paul", "Paul", "Paul", "Paul",
        ]
        .map(|c| Cell::String(c.into()))
        .into()
    }

    fn task_row() -> Vec<Cell> {
        ["A", "B", "C", "D", "E", "F", "G", "H", "I", "G"]
            .map(|c| Cell::String(c.into()))
            .into()
    }

    fn time_row() -> Vec<Cell> {
        [3, 5, 5, 1, 1, 3, 3, 5, 4, 2].map(Cell::Integer).into()
    }

    fn progress_row() -> Vec<Cell> {
        [0.1, 0.5, 0.8, 0.3, 0.9, 0.6, 0.2, 0.7, 0.4, 1.0]
            .map(Cell::Float)
            .into()
    }

    fn budget_row() -> Vec<Cell> {
        [
            50_000, 75_000, 25_000, 150_000, 30_000, 90_000, 45_000, 120_000, 60_000, 200_000,
        ]
        .map(|c| Cell::Decimal(c.into()))
        .into()
    }

    fn due_date_row() -> Vec<Cell> {
        [
            TIMESTAMP + 86400,  // +1 day
            TIMESTAMP + 172800, // +2 days
            TIMESTAMP + 259200, // +3 days
            TIMESTAMP + 345600, // +4 days
            TIMESTAMP + 432000, // +5 days
            TIMESTAMP + 518400, // +6 days
            TIMESTAMP + 604800, // +7 days
            TIMESTAMP + 691200, // +8 days
            TIMESTAMP + 777600, // +9 days
            TIMESTAMP + 864000, // +10 days
        ]
        .map(|c| Cell::DateTime(DateTime::from_timestamp_secs(c).unwrap()))
        .into()
    }

    fn status_row() -> Vec<Cell> {
        [0, 1, 2, 0, 2, 1, 1, 0, 1, 2].map(Cell::Integer).into()
    }

    fn rating_row() -> Vec<Cell> {
        [1, 5, 3, 1, 2, 5, 3, 4, 1, 5].map(Cell::Integer).into()
    }

    fn link_row() -> Vec<Cell> {
        [
            "example.com",
            "example.com",
            "test.com",
            "test.com",
            "staging.com",
            "staging.com",
            "prod.com",
            "prod.com",
            "dev.com",
            "dev.com",
        ]
        .map(|c| Cell::String(c.into()))
        .into()
    }

    fn completed_row() -> Vec<Cell> {
        [
            false, false, true, false, true, false, true, false, false, true,
        ]
        .map(Cell::Boolean)
        .into()
    }

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

        let get_aggregates = |is_ordered: bool, is_numeric: bool| {
            let unordered = [Aggregate::Count].into_iter();
            let ordered = is_ordered
                .then_some([Aggregate::Min, Aggregate::Max])
                .into_iter()
                .flatten();
            let numeric = is_numeric
                .then_some([Aggregate::Sum, Aggregate::Average])
                .into_iter()
                .flatten();
            unordered.chain(ordered).chain(numeric).collect_vec()
        };

        let mut columns = Vec::new();
        for (name, field_kind, row, (is_ordered, is_numeric)) in [
            (
                "Member",
                FieldKind::Text { is_required: true },
                member_row(),
                (false, false),
            ),
            (
                "Task",
                FieldKind::Text { is_required: true },
                task_row(),
                (true, false),
            ),
            (
                "Time",
                FieldKind::Integer {
                    is_required: true,
                    range_start: None,
                    range_end: None,
                },
                time_row(),
                (true, true),
            ),
            (
                "Progress",
                FieldKind::Float {
                    is_required: true,
                    range_start: None,
                    range_end: None,
                },
                progress_row(),
                (true, true),
            ),
            (
                "Budget",
                FieldKind::Money {
                    is_required: true,
                    range_start: None,
                    range_end: None,
                },
                budget_row(),
                (true, true),
            ),
            (
                "Due Date",
                FieldKind::DateTime {
                    is_required: true,
                    range_start: None,
                    range_end: None,
                },
                due_date_row(),
                (true, false),
            ),
            (
                "Status",
                FieldKind::Enumeration {
                    is_required: true,
                    values: HashMap::from_iter([
                        (0, "Scheduled".into()),
                        (1, "In Progress".into()),
                        (2, "Completed".into()),
                    ]),
                    default_value: 0,
                },
                status_row(),
                (true, false),
            ),
            (
                "Rating",
                FieldKind::Progress { total_steps: 5 },
                rating_row(),
                (true, true),
            ),
            (
                "Link",
                FieldKind::WebLink { is_required: true },
                link_row(),
                (false, false),
            ),
            (
                "Completed",
                FieldKind::Checkbox,
                completed_row(),
                (false, false),
            ),
        ] {
            columns.push((
                FieldMetadata::from_field(
                    db::create_field(
                        &db,
                        table_id,
                        CreateField {
                            name: name.into(),
                            field_kind,
                        },
                    )
                    .await?,
                ),
                row,
                get_aggregates(is_ordered, is_numeric),
            ));
        }

        let entries = {
            let mut rows = Vec::new();
            for idx in 0..ROW_COUNT {
                let row = columns.iter().map(|(_, r, _)| r[idx].clone()).collect_vec();
                rows.push(row);
            }
            rows
        };
        let _entries = db::create_entries(
            &db,
            table_id,
            None,
            columns.iter().map(|(f, _, _)| f).cloned().collect(),
            entries,
        )
        .await?;

        let chart_id = db::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id,
                name: "Test".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?
        .chart_id;

        let group_by_column = columns.remove(0);

        let mut create_axes_1 = vec![CreateAxis {
            field_id: group_by_column.0.field_id,
            axis_kind: AxisKind::X,
            aggregate: None,
        }];

        for (field, _, aggregates) in &columns {
            for aggregate in aggregates {
                create_axes_1.push(CreateAxis {
                    field_id: field.field_id,
                    axis_kind: AxisKind::Y,
                    aggregate: Some(*aggregate),
                });
            }
        }

        let axes = super::set_axes(&db, chart_id, table_id, create_axes_1.clone()).await?;
        let create_axes_2: HashSet<_> = axes
            .iter()
            .map(|a| CreateAxis {
                field_id: a.field_id,
                axis_kind: a.axis_kind,
                aggregate: a.aggregate,
            })
            .collect();
        assert_eq!(create_axes_1.len(), create_axes_2.len());
        for create_axis in &create_axes_1 {
            assert!(create_axes_2.contains(create_axis));
        }

        // More to do here

        Ok(())
    }
}

use crate::{
    Id, db,
    model::{
        Cell,
        data::{
            CreateField, Field, FieldIdentifier, FieldKind, FieldMetadata, TableIdentifier,
            UpdateField,
        },
        viz::CreateAxis,
    },
};
use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder, Row, types::Json};
use std::{collections::HashMap, mem::discriminant};

/// Add a field to this table and add a column to the actual SQL table.
pub async fn create_field(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    CreateField { name, field_kind }: CreateField,
) -> sqlx::Result<Field> {
    let mut tx = conn.begin().await?;

    let field: Field = sqlx::query_as(
        r#"
            INSERT INTO meta_field (table_id, name, field_kind)
            VALUES ($1, $2, $3)
            RETURNING *
        "#,
    )
    .bind(table_id)
    .bind(name)
    .bind(sqlx::types::Json(field_kind.clone()))
    .fetch_one(tx.as_mut())
    .await?;

    let column_type = field_kind.get_sql_column();
    let table_ident = TableIdentifier::new(table_id, "data_table");
    let field_ident = FieldIdentifier::new(field.field_id);

    sqlx::query(&format!(
        r#"
            ALTER TABLE {table_ident}
            ADD COLUMN {field_ident} {column_type}
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(field)
}

/// Add fields to this table and add columns the actual SQL table.
pub async fn create_fields(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    fields: Vec<CreateField>,
) -> sqlx::Result<Vec<Field>> {
    let mut tx = conn.begin().await?;

    let fields: Vec<Field> =
        QueryBuilder::new(r#"INSERT INTO meta_field (table_id, name, field_kind)"#)
            .push_values(fields, |mut builder, field| {
                builder
                    .push_bind(table_id)
                    .push_bind(field.name)
                    .push_bind(Json(field.field_kind));
            })
            .push(r#" RETURNING *"#)
            .build_query_as()
            .fetch_all(tx.as_mut())
            .await?;

    let add_column_statement = fields
        .iter()
        .map(|field| {
            let column_type = field.field_kind.0.get_sql_column();
            let field_ident = FieldIdentifier::new(field.field_id);
            format!(r#"ADD COLUMN {field_ident} {column_type}"#)
        })
        .join(", ");

    let table_ident = TableIdentifier::new(table_id, "data_table");

    sqlx::query(&format!(
        r#"
            ALTER TABLE {table_ident}
            {add_column_statement}
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(fields)
}

/// Update a field in this table and change the column in the actual SQL table.
/// This will create a new field and keep the old one as backup if the [FieldKind]
/// variant is different.
pub async fn update_field(
    conn: impl Acquire<'_, Database = Postgres>,
    field_id: Id,
    UpdateField { name, field_kind }: UpdateField,
) -> sqlx::Result<Field> {
    let mut tx = conn.begin().await?;

    let Json(old_field_kind): Json<FieldKind> = sqlx::query_scalar(
        r#"
            SELECT field_kind
            FROM meta_field
            WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    let mut field: Field = sqlx::query_as(
        r#"
            UPDATE meta_field
            SET name = $1, field_kind = $2
            WHERE field_id = $3
            RETURNING *
        "#,
    )
    .bind(name)
    .bind(Json(field_kind.clone()))
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    if discriminant(&field_kind) != discriminant(&old_field_kind) {
        field = convert_field_kind(tx.as_mut(), field, old_field_kind).await?;
    }

    tx.commit().await?;

    Ok(field)
}

/// Create a new field with all the cells converted to the new [FieldKind].
/// Renames the old field to avoid conflict.
async fn convert_field_kind(
    conn: impl Acquire<'_, Database = Postgres>,
    field: Field,
    old_field_kind: FieldKind,
) -> sqlx::Result<Field> {
    let mut tx = conn.begin().await?;

    delete_field_axes(tx.as_mut(), field.field_id).await?;

    let field_ident = FieldIdentifier::new(field.field_id);
    let table_ident = TableIdentifier::new(field.table_id, "data_table");
    let rows = sqlx::query(&format!(
        r#"
            SELECT entry_id, {field_ident}
            FROM {table_ident}
        "#
    ))
    .fetch_all(tx.as_mut())
    .await?;

    let cells: Vec<(Id, Cell)> = rows
        .into_iter()
        .map(|row| {
            let cell = Cell::from_field_row(&row, &field_ident.unquote(), &old_field_kind)?;
            Ok((
                row.get("entry_id"),
                cell.convert_field_kind(&field.field_kind.0)
                    .unwrap_or(Cell::Null),
            ))
        })
        .collect::<sqlx::Result<_>>()?;

    sqlx::query(
        r#"
            UPDATE meta_field
            SET name = name || ' (BACKUP)', field_kind = $1
            WHERE field_id = $2
        "#,
    )
    .bind(Json(old_field_kind))
    .bind(field.field_id)
    .execute(tx.as_mut())
    .await?;

    let mut new_field = create_field(
        tx.as_mut(),
        field.table_id,
        CreateField {
            name: field.name,
            field_kind: field.field_kind.0,
        },
    )
    .await?;

    let cells = cells
        .into_iter()
        .filter(|(_, cell)| !matches!(cell, Cell::Null))
        .collect_vec();

    if !cells.is_empty() {
        let field_ident = FieldIdentifier::new(new_field.field_id);
        QueryBuilder::new(format!(
            r#"
                UPDATE {table_ident}
                SET {field_ident} = data.cell
                FROM (
            "#
        ))
        .push_values(cells, |mut builder, (id, cell)| {
            builder.push_bind(id);
            cell.push_bind(&mut builder);
        })
        .push(format!(
            r#"
                ) AS data (entry_id, cell)
                WHERE {table_ident}.entry_id = data.entry_id
            "#
        ))
        .build()
        .execute(tx.as_mut())
        .await?;
    }

    sqlx::query(
        r#"
            UPDATE meta_field
            SET ordering = CASE field_id
                WHEN $1 THEN (SELECT ordering FROM meta_field WHERE field_id = $2)
                WHEN $2 THEN (SELECT ordering FROM meta_field WHERE field_id = $1)
            END
            WHERE field_id IN ($1, $2);
        "#,
    )
    .bind(field.field_id)
    .bind(new_field.field_id)
    .execute(tx.as_mut())
    .await?;

    let ordering = sqlx::query_scalar(
        r#"
            SELECT ordering
            FROM meta_field
            WHERE field_id = $1
        "#
    ).bind(new_field.field_id).fetch_one(tx.as_mut()).await?;
    new_field.ordering = ordering;

    tx.commit().await?;
    Ok(new_field)
}

/// Delete this field and remove the column from the actual SQL table.
pub async fn delete_field(
    conn: impl Acquire<'_, Database = Postgres>,
    field_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    delete_field_axes(tx.as_mut(), field_id).await?;

    let table_id: Id = sqlx::query_scalar(
        r#"
            DELETE FROM meta_field
            WHERE field_id = $1
            RETURNING table_id
        "#,
    )
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    let table_ident = TableIdentifier::new(table_id, "data_table");
    let field_ident = FieldIdentifier::new(field_id);

    sqlx::query(&format!(
        r#"
            ALTER TABLE {table_ident}
            DROP COLUMN {field_ident}
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
}

async fn delete_field_axes(
    conn: impl Acquire<'_, Database = Postgres>,
    field_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    let table_id: Id = sqlx::query_scalar(
        r#"
            SELECT table_id
            FROM meta_field
            WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    let affected_chart_ids: Vec<Id> = sqlx::query_scalar(
        r#"
            SELECT DISTINCT c.chart_id
            FROM chart AS c
            JOIN axis AS a
            ON c.chart_id = a.chart_id
            WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_all(tx.as_mut())
    .await?;

    sqlx::query(
        r#"
        DELETE FROM axis
        WHERE axis_id = $1
    "#,
    )
    .bind(field_id)
    .execute(tx.as_mut())
    .await?;

    for chart_id in affected_chart_ids {
        let axes: Vec<CreateAxis> = sqlx::query_as(
            r#"
                SELECT *
                FROM axis
                WHERE chart_id = $1
            "#,
        )
        .bind(chart_id)
        .fetch_all(tx.as_mut())
        .await?;

        db::set_axes(tx.as_mut(), chart_id, table_id, axes).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Get all fields of this table.
pub async fn get_fields(executor: impl PgExecutor<'_>, table_id: Id) -> sqlx::Result<Vec<Field>> {
    sqlx::query_as(
        r#"
            SELECT *
            FROM meta_field
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await
}

/// Get all field IDs of this table.
pub async fn get_field_ids(executor: impl PgExecutor<'_>, table_id: Id) -> sqlx::Result<Vec<Id>> {
    sqlx::query_scalar(
        r#"
            SELECT field_id
            FROM meta_field
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await
}

/// Set the order of all fields in this table.
pub async fn set_field_order(
    conn: impl Acquire<'_, Database = Postgres>,
    order: HashMap<Id, i32>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(
        r#"
            UPDATE meta_field AS f
            SET ordering = n.ordering
            FROM (
                SELECT
                    unnest($1::int[]) AS field_id,
                    unnest($2::int[]) AS ordering
            ) AS n
            WHERE f.field_id = n.field_id
        "#,
    )
    .bind(order.keys().collect_vec())
    .bind(order.values().collect_vec())
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Get the all [FieldMetadata] of this table.
pub async fn get_fields_metadata(
    executor: impl PgExecutor<'_>,
    table_id: Id,
) -> sqlx::Result<Vec<FieldMetadata>> {
    sqlx::query_as(
        r#"
            SELECT
                field_id,
                field_kind
            FROM meta_field
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await
}

pub async fn field_exists(
    executor: impl PgExecutor<'_>,
    table_id: Id,
    field_id: Id,
) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM meta_field
                WHERE table_id = $1 AND field_id = $2
            )
        "#,
    )
    .bind(table_id)
    .bind(field_id)
    .fetch_one(executor)
    .await
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        Id, db,
        model::{
            Cell,
            data::{
                CreateField, CreateTable, Field, FieldIdentifier, FieldKind, FieldMetadata,
                TableIdentifier, UpdateField,
            },
            viz::{
                Aggregate, AxisIdentifier, AxisKind, ChartIdentifier, ChartKind, CreateAxis,
                CreateChart, CreateDashboard,
            },
        },
        test_util,
    };
    use itertools::Itertools;
    use sqlx::{PgPool, types::Json};
    use std::collections::HashMap;

    #[sqlx::test]
    async fn create_field(db: PgPool) -> anyhow::Result<()> {
        
        for (idx, (field_kind, test_value)) in test_util::field_tests().into_iter().enumerate() {
            let table_id = db::create_table(
                &db,
                CreateTable {
                    parent_id: None,
                    name: "test".into(),
                    description: "".into(),
                },
            )
            .await?
            .table_id;
            let create_field = CreateField {
                name: idx.to_string(),
                field_kind: field_kind.clone(),
            };
            let field_1 = super::create_field(&db, table_id, create_field.clone()).await?;
            assert_eq!(create_field.name, field_1.name);
            assert_eq!(create_field.field_kind, field_1.field_kind.0);
            let field_2 = sqlx::query_as(r#"SELECT * FROM meta_field WHERE field_id = $1"#)
                .bind(field_1.field_id)
                .fetch_one(&db)
                .await?;
            assert_eq!(field_1, field_2);
            assert!(
                test_util::test_insert_cell(&db, table_id, field_1.field_id, test_value,).await
            );
        }
        Ok(())
    }

    #[sqlx::test]
    async fn create_fields(db: PgPool) -> anyhow::Result<()> {
        
        let field_kind_tests = test_util::field_tests()
            .into_iter()
            .enumerate()
            .map(|(idx, f)| (idx.to_string(), f))
            .collect_vec();
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let create_fields_1 = field_kind_tests
            .iter()
            .map(|(name, (field_kind, _))| CreateField {
                name: name.clone(),
                field_kind: field_kind.clone(),
            })
            .collect_vec();
        let fields = super::create_fields(&db, table_id, create_fields_1.clone()).await?;

        let field_ids = fields
            .iter()
            .map(|f| (f.name.clone(), f.field_id))
            .collect::<HashMap<_, _>>();
        let create_fields_2 = fields
            .into_iter()
            .map(|f| CreateField {
                name: f.name,
                field_kind: f.field_kind.0,
            })
            .collect_vec();
        test_util::assert_eq_vec(create_fields_1.clone(), create_fields_2, |f| f.name.clone());

        let create_fields_3 = sqlx::query_as::<_, (String, Json<FieldKind>)>(
            r#"SELECT name, field_kind FROM meta_field WHERE table_id = $1"#,
        )
        .bind(table_id)
        .fetch_all(&db)
        .await?
        .into_iter()
        .map(|(name, field_kind)| CreateField {
            name,
            field_kind: field_kind.0,
        })
        .collect_vec();

        test_util::assert_eq_vec(create_fields_1, create_fields_3, |f| f.name.clone());
        for (name, (_, test_value)) in field_kind_tests {
            assert!(
                test_util::test_insert_cell(
                    &db,
                    table_id,
                    *field_ids.get(&name).unwrap(),
                    test_value,
                )
                .await
            );
        }
        Ok(())
    }

    #[sqlx::test]
    async fn update_field(db: PgPool) -> anyhow::Result<()> {
        
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let create_field = CreateField {
            name: "test".into(),
            field_kind: FieldKind::Integer {
                is_required: false,
                range_start: Some(10),
                range_end: Some(40),
            },
        };

        // Test changing field metadata
        let field_id = super::create_field(&db, table_id, create_field)
            .await?
            .field_id;
        let update_field = UpdateField {
            name: "different".into(),
            field_kind: FieldKind::Integer {
                is_required: true,
                range_start: Some(-99),
                range_end: Some(99),
            },
        };
        let field_1 = super::update_field(&db, field_id, update_field.clone()).await?;
        assert_eq!(update_field.name, field_1.name);
        assert_eq!(update_field.field_kind, field_1.field_kind.0);
        let field_2: Field = sqlx::query_as(r#"SELECT * FROM meta_field WHERE field_id = $1"#)
            .bind(field_1.field_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(field_1, field_2);

        // Test converting field kind
        let update_field = UpdateField {
            name: "converted".into(),
            field_kind: FieldKind::Text { is_required: false },
        };
        let field_1 = super::update_field(&db, field_id, update_field.clone()).await?;
        assert_eq!(update_field.name, field_1.name);
        assert_eq!(update_field.field_kind, field_1.field_kind.0);
        let mut field_2: Field = sqlx::query_as(r#"SELECT * FROM meta_field WHERE field_id = $1"#)
            .bind(field_1.field_id)
            .fetch_one(&db)
            .await?;
        field_2.updated_at = None;
        assert_eq!(field_1, field_2);
        Ok(())
    }

    #[sqlx::test]
    async fn convert_field_kind(db: PgPool) -> anyhow::Result<()> {
        // Test the following converts:
        // to Integer
        // to Float
        // to Money
        // to DateTime
        // to Checkbox
        // to Enumeration
        // Only going to test conversions from Text as it has the most value and risk

        let old_field_kind = FieldKind::Text { is_required: false };
        for (idx, (field_kind, new_value)) in test_util::field_tests().into_iter().enumerate() {
            let old_value = Cell::String(match field_kind.clone() {
                FieldKind::Text { .. } => continue,
                FieldKind::Enumeration { values, .. } => {
                    let idx = match new_value {
                        Cell::Integer(v) => v,
                        _ => panic!(),
                    };
                    values[&idx].clone()
                }
                _ => new_value.to_string(),
            });
            println!(
                "{:?}: old_value: {:?} new_value {:?}",
                field_kind, old_value, new_value
            );

            let table_id = db::create_table(
                &db,
                CreateTable {
                    parent_id: None,
                    name: "test".into(),
                    description: "".into(),
                },
            )
            .await?
            .table_id;
            let old_field_1 = super::create_field(
                &db,
                table_id,
                CreateField {
                    name: idx.to_string(),
                    field_kind: old_field_kind.clone(),
                },
            )
            .await?;
            test_util::test_insert_cell(&db, table_id, old_field_1.field_id, old_value.clone())
                .await;
            println!("OK");
            let new_field_1: Field = sqlx::query_as(
                r#"UPDATE meta_field SET field_kind = $1 WHERE field_id = $2 RETURNING *"#,
            )
            .bind(Json(field_kind.clone()))
            .bind(old_field_1.field_id)
            .fetch_one(&db)
            .await?;
            println!("OK");
            let mut new_field_2 =
                super::convert_field_kind(&db, new_field_1.clone(), old_field_kind.clone()).await?;
            assert_eq!(new_field_1.name, new_field_2.name);
            assert_eq!(new_field_1.field_kind, new_field_2.field_kind);

            let table_ident = TableIdentifier::new(table_id, "data_table");
            let field_ident = FieldIdentifier::new(new_field_2.field_id);
            let actual_new_value = Cell::from_field_row(
                &sqlx::query(&format!(r#"SELECT {field_ident} FROM {table_ident}"#))
                    .fetch_one(&db)
                    .await?,
                &field_ident.unquote(),
                &field_kind,
            )?;
            assert_eq!(new_value, actual_new_value);

            let old_field_2: Field =
                sqlx::query_as(r#"SELECT * FROM meta_field WHERE field_id = $1"#)
                    .bind(old_field_1.field_id)
                    .fetch_one(&db)
                    .await?;
            let mut new_field_3: Field =
                sqlx::query_as(r#"SELECT * FROM meta_field WHERE field_id = $1"#)
                    .bind(new_field_2.field_id)
                    .fetch_one(&db)
                    .await?;
            assert_ne!(old_field_1.name, old_field_2.name);
            assert_eq!(old_field_1.field_kind, old_field_2.field_kind);
            new_field_3.updated_at = None;
            assert_eq!(new_field_2, new_field_3);

            assert!(
                test_util::test_insert_cell(&db, table_id, old_field_1.field_id, old_value).await
            );
            assert!(
                test_util::test_insert_cell(&db, table_id, new_field_2.field_id, new_value.clone())
                    .await
            );
        }
        Ok(())
    }

    #[sqlx::test]
    async fn delete_field(db: PgPool) -> anyhow::Result<()> {
        
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let field_id = super::create_field(
            &db,
            table_id,
            CreateField {
                name: "test".into(),
                field_kind: FieldKind::Checkbox,
            },
        )
        .await?
        .field_id;

        super::delete_field(&db, field_id).await?;
        let not_exists: bool = sqlx::query_scalar(
            r#"SELECT NOT EXISTS (SELECT 1 FROM meta_field WHERE field_id = $1)"#,
        )
        .bind(field_id)
        .fetch_one(&db)
        .await?;
        assert!(not_exists);
        assert!(!test_util::test_insert_cell(&db, table_id, field_id, Cell::Boolean(true)).await);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_field_axes(db: PgPool) -> anyhow::Result<()> {
        
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let field_id = db::create_field(
            &db,
            table_id,
            CreateField {
                name: "test".into(),
                field_kind: FieldKind::Checkbox,
            },
        )
        .await?
        .field_id;
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let chart_id = db::create_chart(
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
                aggregate: Some(Aggregate::Count),
            }],
        )
        .await?
        .into_iter()
        .next()
        .unwrap()
        .axis_id;

        super::delete_field_axes(&db, field_id).await?;
        let not_exists: bool =
            sqlx::query_scalar(r#"SELECT NOT EXISTS (SELECT 1 FROM axis WHERE axis_id = $1)"#)
                .bind(axis_id)
                .fetch_one(&db)
                .await?;
        assert!(not_exists);

        let chart_ident = ChartIdentifier::new(chart_id, "data_view");
        let axis_ident = AxisIdentifier::new(axis_id);
        let column_not_found =
            sqlx::query_scalar::<_, i32>(&format!(r#"SELECT {chart_ident} FROM {axis_ident}"#))
                .fetch_all(&db)
                .await
                .unwrap_err();
        assert!(matches!(column_not_found, sqlx::Error::Database(_)));
        Ok(())
    }

    #[sqlx::test]
    async fn get_fields(db: PgPool) -> anyhow::Result<()> {
        
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let mut fields_1: Vec<Field> = Vec::new();
        for (idx, field_kind) in [
            FieldKind::Checkbox,
            FieldKind::Text { is_required: false },
            FieldKind::DateTime {
                is_required: false,
                range_start: None,
                range_end: None,
            },
        ]
        .into_iter()
        .enumerate()
        {
            fields_1.push(
                super::create_field(
                    &db,
                    table_id,
                    CreateField {
                        name: idx.to_string(),
                        field_kind,
                    },
                )
                .await?,
            );
        }
        let fields_2 = super::get_fields(&db, table_id).await?;
        test_util::assert_eq_vec(fields_1, fields_2, |f| f.field_id);
        Ok(())
    }

    #[sqlx::test]
    async fn get_field_ids(db: PgPool) -> anyhow::Result<()> {
        
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let mut field_ids_1: Vec<Id> = Vec::new();
        for (idx, field_kind) in [
            FieldKind::Checkbox,
            FieldKind::Text { is_required: false },
            FieldKind::DateTime {
                is_required: false,
                range_start: None,
                range_end: None,
            },
        ]
        .into_iter()
        .enumerate()
        {
            field_ids_1.push(
                super::create_field(
                    &db,
                    table_id,
                    CreateField {
                        name: idx.to_string(),
                        field_kind,
                    },
                )
                .await?
                .field_id,
            );
        }
        let field_ids_2 = super::get_field_ids(&db, table_id).await?;
        test_util::assert_eq_vec(field_ids_1, field_ids_2, |f| *f);
        Ok(())
    }

    #[sqlx::test]
    async fn set_field_order(db: PgPool) -> anyhow::Result<()> {
        
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let mut fields_1: Vec<Field> = Vec::new();
        for (idx, field_kind) in [
            FieldKind::Checkbox,
            FieldKind::Text { is_required: false },
            FieldKind::DateTime {
                is_required: false,
                range_start: None,
                range_end: None,
            },
        ]
        .into_iter()
        .enumerate()
        {
            fields_1.push(
                super::create_field(
                    &db,
                    table_id,
                    CreateField {
                        name: idx.to_string(),
                        field_kind,
                    },
                )
                .await?,
            );
        }
        let initial_ordering = fields_1.iter().map(|f| f.ordering).collect_vec();
        assert_eq!(vec![0, 1, 2], initial_ordering);
        let field_ordering_1 = fields_1
            .iter()
            .zip(initial_ordering.into_iter().rev())
            .map(|(f, ordering)| (f.field_id, ordering))
            .collect_vec();
        super::set_field_order(&db, HashMap::from_iter(field_ordering_1.clone())).await?;
        let field_ordering_2: Vec<(Id, i32)> = sqlx::query_as(
            r#"SELECT field_id, ordering FROM meta_field WHERE table_id = $1 ORDER BY field_id"#,
        )
        .bind(table_id)
        .fetch_all(&db)
        .await?;
        assert_eq!(field_ordering_1, field_ordering_2);
        Ok(())
    }

    #[sqlx::test]
    async fn get_fields_metadata(db: PgPool) -> anyhow::Result<()> {
        
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        let mut fields_metadata_1: Vec<FieldMetadata> = Vec::new();
        for (idx, field_kind) in [
            FieldKind::Checkbox,
            FieldKind::Text { is_required: false },
            FieldKind::DateTime {
                is_required: false,
                range_start: None,
                range_end: None,
            },
        ]
        .into_iter()
        .enumerate()
        {
            let field = super::create_field(
                &db,
                table_id,
                CreateField {
                    name: idx.to_string(),
                    field_kind,
                },
            )
            .await?;
            fields_metadata_1.push(FieldMetadata {
                field_id: field.field_id,
                field_kind: field.field_kind,
            });
        }
        let fields_metadata_2 = super::get_fields_metadata(&db, table_id).await?;
        test_util::assert_eq_vec(fields_metadata_1, fields_metadata_2, |f| f.field_id);
        Ok(())
    }

    #[sqlx::test]
    async fn field_exists(db: PgPool) -> anyhow::Result<()> {
        
        let table_id = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .table_id;
        assert!(!super::field_exists(&db, table_id, 1).await?);
        let field = super::create_field(
            &db,
            table_id,
            CreateField {
                name: "test".into(),
                field_kind: FieldKind::Checkbox,
            },
        )
        .await?;
        assert!(super::field_exists(&db, table_id, field.field_id).await?);
        Ok(())
    }
}

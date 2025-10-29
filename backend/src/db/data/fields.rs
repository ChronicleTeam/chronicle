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

    let column_type = field_kind.get_sql_type();
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

    return Ok(field);
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
            .push(
                r#" RETURNING *"#,
            )
            .build_query_as()
            .fetch_all(tx.as_mut())
            .await?;

    let add_column_statement = fields
        .iter()
        .map(|field| {
            let column_type = field.field_kind.0.get_sql_type();
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

    return Ok(fields);
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

    let field = create_field(
        tx.as_mut(),
        field.table_id,
        CreateField {
            name: field.name,
            field_kind: field.field_kind.0,
        },
    )
    .await?;

    let field_ident = FieldIdentifier::new(field.field_id);

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

    tx.commit().await?;

    Ok(field)
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
                WHERE table_id = $1 field_id = $2
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
    use std::collections::HashMap;

    use itertools::Itertools;
    use sqlx::{PgPool, types::Json};

    use crate::{
        db,
        model::data::{CreateField, CreateTable, FieldKind},
        test_util,
    };

    // TODO: consider testing the generated DDL
    #[sqlx::test]
    async fn create_field(db: PgPool) -> anyhow::Result<()> {
        for (idx, field_kind_test) in test_util::field_kind_tests().into_iter().enumerate() {
            let table = db::create_table(
                &db,
                CreateTable {
                    parent_id: None,
                    name: "test".into(),
                    description: "".into(),
                },
            )
            .await?;
            let create_field = CreateField {
                name: format!("Field {}", idx),
                field_kind: field_kind_test.field_kind.clone(),
            };
            let field_1 = super::create_field(&db, table.table_id, create_field.clone()).await?;
            assert_eq!(create_field.name, field_1.name);
            assert_eq!(create_field.field_kind, field_1.field_kind.0);
            let field_2 = sqlx::query_as(r#"SELECT * FROM meta_field WHERE field_id = $1"#)
                .bind(field_1.field_id)
                .fetch_one(&db)
                .await?;
            assert_eq!(field_1, field_2);
            field_kind_test
                .test_insert(&db, table.table_id, field_1.field_id)
                .await;
        }
        Ok(())
    }

    // TODO: consider testing the generated DDL
    #[sqlx::test]
    async fn create_fields(db: PgPool) -> anyhow::Result<()> {
        let field_kind_tests = test_util::field_kind_tests()
            .into_iter()
            .enumerate()
            .map(|(idx, f)| (format!("Field {}", idx), f))
            .collect_vec();
        let table = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?;
        let create_fields_1 = field_kind_tests
            .iter()
            .map(|(name, f)| CreateField {
                name: name.clone(),
                field_kind: f.field_kind.clone(),
            })
            .collect_vec();
        let fields = super::create_fields(&db, table.table_id, create_fields_1.clone()).await?;

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
        .bind(table.table_id)
        .fetch_all(&db)
        .await?
        .into_iter()
        .map(|(name, field_kind)| CreateField {
            name,
            field_kind: field_kind.0,
        })
        .collect_vec();

        test_util::assert_eq_vec(create_fields_1, create_fields_3, |f| f.name.clone());
        for (name, field_kind_test) in field_kind_tests {
            field_kind_test.test_insert(&db, table.table_id, *field_ids.get(&name).unwrap()).await;
        }
        Ok(())
    }

    // TODO: consider testing the generated DDL
    #[sqlx::test]
    async fn update_field(db: PgPool) -> anyhow::Result<()> {
         let table = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?;
        
        todo!()
    }

    // TODO: consider testing the generated DDL
    #[sqlx::test]
    async fn convert_field_kind(db: PgPool) -> anyhow::Result<()> {
        todo!()
    }

    // TODO: consider testing the generated DDL
    #[sqlx::test]
    async fn delete_field(db: PgPool) -> anyhow::Result<()> {
        todo!()
    }

    #[sqlx::test]
    async fn delete_field_axes(db: PgPool) -> anyhow::Result<()> {
        todo!()
    }

    #[sqlx::test]
    async fn get_fields(db: PgPool) -> anyhow::Result<()> {
        todo!()
    }

    #[sqlx::test]
    async fn get_field_ids(db: PgPool) -> anyhow::Result<()> {
        todo!()
    }

    #[sqlx::test]
    async fn set_field_order(db: PgPool) -> anyhow::Result<()> {
        todo!()
    }

    #[sqlx::test]
    async fn get_fields_metadata(db: PgPool) -> anyhow::Result<()> {
        todo!()
    }

    #[sqlx::test]
    async fn field_exists(db: PgPool) -> anyhow::Result<()> {
        todo!()
    }
}

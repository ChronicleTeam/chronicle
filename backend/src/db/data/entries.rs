use super::{entry_from_row, insert_columns, select_columns, update_columns};
use crate::{
    Id,
    model::{
        Cell,
        data::{Entry, FieldIdentifier, FieldMetadata, TableIdentifier},
    },
};
use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

/// Add entries to the actual SQL table.
pub async fn create_entries(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    parent_id: Option<Id>,
    fields: Vec<FieldMetadata>,
    entries: Vec<Vec<Cell>>,
) -> sqlx::Result<Vec<Entry>> {
    let mut tx = conn.begin().await?;

    let table_ident = TableIdentifier::new(table_id, "data_table");

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let insert_columns = insert_columns(parent_id.is_some(), &field_idents);
    let return_columns = select_columns(parent_id.is_some(), &field_idents);

    let rows = QueryBuilder::new(format!(r#"INSERT INTO {table_ident} ({insert_columns})"#))
        .push_values(entries, |mut builder, entry| {
            for cell in entry {
                cell.push_bind(&mut builder);
            }
            if let Some(parent_id) = parent_id {
                builder.push_bind(parent_id);
            }
        })
        .push(format!(
            r#"
                RETURNING {return_columns}
            "#
        ))
        .build()
        .fetch_all(tx.as_mut())
        .await?;

    let entries = rows
        .into_iter()
        .map(|row| entry_from_row(row, &fields).unwrap())
        .collect_vec();

    tx.commit().await?;

    Ok(entries)
}

/// Update an entry in the actual SQL table.
pub async fn update_entry(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    entry_id: Id,
    parent_id: Option<Id>,
    fields: Vec<FieldMetadata>,
    entry: Vec<Cell>,
) -> sqlx::Result<Entry> {
    let mut tx = conn.begin().await?;

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let set_columns = update_columns(parent_id.is_some(), &field_idents, 2);

    let return_columns = select_columns(parent_id.is_some(), &field_idents);

    let table_ident = TableIdentifier::new(table_id, "data_table");

    let update_query = format!(
        r#"
            UPDATE {table_ident}
            SET {set_columns}
            WHERE entry_id = $1
            RETURNING {return_columns}
        "#,
    );
    let mut update_query = sqlx::query(&update_query).bind(entry_id);

    for cell in entry {
        update_query = cell.bind(update_query);
    }
    if let Some(parent_id) = parent_id {
        update_query = update_query.bind(parent_id);
    }

    let entry = entry_from_row(update_query.fetch_one(tx.as_mut()).await?, &fields)?;

    tx.commit().await?;

    Ok(entry)
}

/// Delete an entry in the actual SQL table.
pub async fn delete_entry(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    entry_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let table_ident = TableIdentifier::new(table_id, "data_table");
    sqlx::query(&format!(
        r#"
            DELETE FROM {table_ident}
            WHERE entry_id = $1
        "#
    ))
    .bind(entry_id)
    .execute(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Check whether the entry exists.
pub async fn entry_exists(
    executor: impl PgExecutor<'_>,
    table_id: Id,
    entry_id: Id,
) -> sqlx::Result<bool> {
    let table_ident = TableIdentifier::new(table_id, "data_table");
    sqlx::query_scalar(&format!(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM {table_ident}
                WHERE entry_id = $1
            )
        "#,
    ))
    .bind(entry_id)
    .fetch_one(executor)
    .await
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        db::{
            self,
            data::{entry_from_row, select_columns},
        },
        model::data::{CreateField, CreateTable, FieldIdentifier, FieldMetadata, TableIdentifier},
        test_util,
    };
    use itertools::Itertools;
    use sqlx::PgPool;
    use std::iter;

    #[sqlx::test]
    async fn create_entries(db: PgPool) -> anyhow::Result<()> {
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

        let mut fields: Vec<FieldMetadata> = Vec::new();
        let (field_kinds, entry): (Vec<_>, Vec<_>) = test_util::field_tests().into_iter().unzip();
        for (idx, field_kind) in field_kinds.into_iter().enumerate() {
            let field = db::create_field(
                &db,
                table_id,
                CreateField {
                    name: idx.to_string(),
                    field_kind,
                },
            )
            .await?;
            fields.push(FieldMetadata {
                field_id: field.field_id,
                field_kind: field.field_kind,
            });
        }

        let entries_1 = iter::repeat_n(entry, 3).collect_vec();
        let parent_id = None;
        let entries_2 =
            super::create_entries(&db, table_id, parent_id, fields.clone(), entries_1.clone())
                .await?;
        let field_ids = fields.iter().map(|f| f.field_id).collect_vec();
        let entries_2_fmt = entries_2
            .iter()
            .map(|e| {
                field_ids
                    .iter()
                    .map(|id| e.cells.get(id).unwrap().clone())
                    .collect_vec()
            })
            .collect_vec();
        assert_eq!(entries_1, entries_2_fmt,);
        assert!(entries_2.iter().all(|e| e.parent_id == parent_id));

        let table_ident = TableIdentifier::new(table_id, "data_table");
        let field_idents = fields
            .iter()
            .map(|field| FieldIdentifier::new(field.field_id))
            .collect_vec();
        let select_columns = select_columns(parent_id.is_some(), &field_idents);
        let entries_3 = sqlx::query(&format!(r#"SELECT {select_columns} FROM {table_ident}"#))
            .fetch_all(&db)
            .await?
            .into_iter()
            .map(|row| entry_from_row(row, &fields).unwrap())
            .collect_vec();
        assert_eq!(entries_2, entries_3);

        Ok(())
    }

    #[sqlx::test]
    async fn update_entry(db: PgPool) -> anyhow::Result<()> {
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

        let mut fields: Vec<FieldMetadata> = Vec::new();
        let (field_kinds, entry_1): (Vec<_>, Vec<_>) = test_util::field_tests().into_iter().unzip();
        for (idx, field_kind) in field_kinds.into_iter().enumerate() {
            let field = db::create_field(
                &db,
                table_id,
                CreateField {
                    name: idx.to_string(),
                    field_kind,
                },
            )
            .await?;
            fields.push(FieldMetadata {
                field_id: field.field_id,
                field_kind: field.field_kind,
            });
        }
        let table_ident = TableIdentifier::new(table_id, "data_table");
        let entry_id = sqlx::query_scalar(&format!(
            r#"INSERT INTO {table_ident} DEFAULT VALUES RETURNING entry_id"#
        ))
        .fetch_one(&db)
        .await?;
        let parent_id = None;
        let entry_2 = super::update_entry(
            &db,
            table_id,
            entry_id,
            parent_id,
            fields.clone(),
            entry_1.clone(),
        )
        .await?;

        let field_ids = fields.iter().map(|f| f.field_id).collect_vec();
        let entry_2_fmt = field_ids
            .iter()
            .map(|id| entry_2.cells.get(id).unwrap().clone())
            .collect_vec();
        assert_eq!(entry_1, entry_2_fmt);
        assert_eq!(entry_id, entry_2.entry_id);
        assert_eq!(parent_id, entry_2.parent_id);

        let field_idents = fields
            .iter()
            .map(|field| FieldIdentifier::new(field.field_id))
            .collect_vec();
        let select_columns = select_columns(parent_id.is_some(), &field_idents);
        let entry_3 = entry_from_row(
            sqlx::query(&format!(
                r#"SELECT {select_columns} FROM {table_ident} WHERE entry_id = $1"#
            ))
            .bind(entry_id)
            .fetch_one(&db)
            .await?,
            &fields,
        )
        .unwrap();
        assert_eq!(entry_2, entry_3);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_entry(db: PgPool) -> anyhow::Result<()> {
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
        let table_ident = TableIdentifier::new(table_id, "data_table");
        let entry_id = sqlx::query_scalar(&format!(
            r#"INSERT INTO {table_ident} DEFAULT VALUES RETURNING entry_id"#
        ))
        .fetch_one(&db)
        .await?;

        super::delete_entry(&db, table_id, entry_id).await?;
        let not_exists: bool = sqlx::query_scalar(&format!(
            r#"SELECT NOT EXISTS (SELECT 1 FROM {table_ident} WHERE entry_id = $1)"#
        ))
        .bind(entry_id)
        .fetch_one(&db)
        .await?;
        assert!(not_exists);
        Ok(())
    }

    #[sqlx::test]
    async fn entry_exists(db: PgPool) -> anyhow::Result<()> {
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
        let table_ident = TableIdentifier::new(table_id, "data_table");
        let entry_id = sqlx::query_scalar(&format!(
            r#"INSERT INTO {table_ident} DEFAULT VALUES RETURNING entry_id"#
        ))
        .fetch_one(&db)
        .await?;

        let exists = super::entry_exists(&db, table_id, entry_id).await?;
        assert!(exists);
        let exists = super::entry_exists(&db, table_id, entry_id + 1).await?;
        assert!(!exists);
        Ok(())
    }
}

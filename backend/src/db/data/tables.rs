use super::{entry_from_row, select_columns};
use crate::{
    Id, db,
    model::{
        access::AccessRole,
        data::{
            CreateTable, Field, FieldIdentifier, FieldMetadata, GetTable, Table, TableData,
            TableIdentifier, UpdateTable,
        },
    },
};
use futures::future::join_all;
use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres};

/// Add a table to this user and create the actual SQL table.
pub async fn create_table(
    conn: impl Acquire<'_, Database = Postgres>,
    CreateTable {
        parent_id,
        name,
        description,
    }: CreateTable,
) -> sqlx::Result<Table> {
    let mut tx = conn.begin().await?;

    let table: Table = sqlx::query_as(
        r#"
            INSERT INTO meta_table (parent_id, name, description)
            VALUES ($1, $2, $3) 
            RETURNING *
        "#,
    )
    .bind(parent_id)
    .bind(name)
    .bind(description)
    .fetch_one(tx.as_mut())
    .await?;

    let table_ident = TableIdentifier::new(table.table_id, "data_table");
    let parent_id_column = if let Some(parent_id) = table.parent_id {
        let parent_ident = TableIdentifier::new(parent_id, "data_table");
        format!("parent_id INT NOT NULL REFERENCES {parent_ident}(entry_id),")
    } else {
        String::new()
    };

    sqlx::query(&format!(
        r#"
            CREATE TABLE {table_ident} (
                entry_id SERIAL PRIMARY KEY,
                {parent_id_column}
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ
            )
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    sqlx::query(&format!(r#"SELECT trigger_updated_at('{table_ident}')"#))
        .execute(tx.as_mut())
        .await?;

    tx.commit().await?;

    Ok(table)
}

/// Update the table metadata.
pub async fn update_table(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    UpdateTable { name, description }: UpdateTable,
) -> sqlx::Result<Table> {
    let mut tx = conn.begin().await?;

    let table = sqlx::query_as(
        r#"
            UPDATE meta_table
            SET name = $1, description = $2
            WHERE table_id = $3
            RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(table_id)
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(table)
}

/// Delete this table along with the actual SQL table and the fields.
pub async fn delete_table(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    let chart_ids: Vec<Id> = sqlx::query_scalar(
        r#"
            SELECT chart_id
            FROM chart
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(tx.as_mut())
    .await?;
    for chart_id in chart_ids {
        db::delete_chart(tx.as_mut(), chart_id).await?;
    }

    sqlx::query(
        r#"
            DELETE FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .execute(tx.as_mut())
    .await?;

    let table_ident = TableIdentifier::new(table_id, "data_table");

    sqlx::query(&format!(r#"DROP TABLE {table_ident} CASCADE"#))
        .execute(tx.as_mut())
        .await?;

    tx.commit().await?;

    Ok(())
}

/// Get the parent ID of this table.
pub async fn get_table_parent_id(
    executor: impl PgExecutor<'_>,
    table_id: Id,
) -> sqlx::Result<Option<Id>> {
    sqlx::query_scalar(
        r#"
            SELECT parent_id
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(executor)
    .await
}

/// Get all tables belonging to this user.
pub async fn get_tables(executor: impl PgExecutor<'_>, user_id: Id) -> sqlx::Result<Vec<GetTable>> {
    sqlx::query_as(
        r#"
            SELECT *
            FROM meta_table AS t
            JOIN meta_table_access_v AS a
            ON t.table_id = a.resource_id
            WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
}

/// Get all the children tables of this table.
pub async fn get_table_children(
    executor: impl PgExecutor<'_>,
    table_id: Id,
) -> sqlx::Result<Vec<Table>> {
    sqlx::query_as(
        r#"
            SELECT *
            FROM meta_table
            WHERE parent_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await
}

/// Get the table, its fields, its entries, and its child tables.
pub async fn get_table_data(
    executor: impl PgExecutor<'_> + Copy,
    table_id: Id,
) -> sqlx::Result<TableData> {
    let table: Table = sqlx::query_as(
        r#"
            SELECT *
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(executor)
    .await?;

    let fields: Vec<Field> = sqlx::query_as(
        r#"
            SELECT *
            FROM meta_field
            WHERE table_id = $1
            ORDER BY field_id
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await?;

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let select_columns = select_columns(table.parent_id.is_some(), &field_idents);

    let table_ident = TableIdentifier::new(table_id, "data_table");
    let entries = sqlx::query::<Postgres>(&format!(
        r#"
            SELECT {select_columns}
            FROM {table_ident}
        "#
    ))
    .fetch_all(executor)
    .await?
    .into_iter()
    .map(|row| {
        entry_from_row(
            row,
            &fields
                .iter()
                .map(
                    |Field {
                         field_id,
                         field_kind,
                         ..
                     }| FieldMetadata {
                        field_id: *field_id,
                        field_kind: field_kind.clone(),
                    },
                )
                .collect_vec(),
        )
    })
    .try_collect()?;

    let children_ids = sqlx::query_scalar(
        r#"
            SELECT table_id
            FROM meta_table
            WHERE parent_id = $1
         "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await?;

    let children = join_all(
        children_ids
            .into_iter()
            .map(|child_id| get_table_data(executor, child_id)),
    )
    .await
    .into_iter()
    .try_collect()?;

    Ok(TableData {
        table,
        fields,
        entries,
        children,
    })
}

pub async fn delete_tables_without_owner(
    conn: impl Acquire<'_, Database = Postgres>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let table_ids: Vec<Id> = sqlx::query_scalar(
        r#"
        SELECT table_id
        FROM meta_table AS t
        WHERE NOT EXISTS (
            SELECT 1
            FROM meta_table_access AS a
            WHERE a.resource_id = t.table_id
            AND a.access_role = $1
        )
    "#,
    )
    .bind(AccessRole::Owner)
    .fetch_all(tx.as_mut())
    .await?;

    for table_id in table_ids {
        delete_table(tx.as_mut(), table_id).await?;
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use anyhow::Ok;
    use sqlx::PgPool;

    use crate::{db, model::data::CreateTable};

    #[sqlx::test]
    async fn test_create_table(db: PgPool) -> anyhow::Result<()> {
        let user = db::create_user(&db, "John".into(), "1234".into(), false).await?;
        let table1 = db::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "table1".into(),
                description: "Table 1".into(),
            },
        )
        .await?;

        let table2 = sqlx::query_as(
            r#"
            SELECT 1
            FROM meta_table AS t
            JOIN meta_table_access_v AS a
            ON t.table_id = a.resource_id
            WHERE user_id = $1
        "#,
        )
        .bind(user.user_id)
        .fetch_one(&db)
        .await?;

        assert_eq!(table1, table2);

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_table(db: PgPool) -> anyhow::Result<()> {
        Ok(())
    }
    #[sqlx::test]
    async fn test_delete_table(db: PgPool) -> anyhow::Result<()> {
        Ok(())
    }
    #[sqlx::test]
    async fn test_get_table_parent(db: PgPool) -> anyhow::Result<()> {
        Ok(())
    }
    #[sqlx::test]
    async fn test_get_tables(db: PgPool) -> anyhow::Result<()> {
        Ok(())
    }

    #[sqlx::test]
    async fn test_get_table_children(db: PgPool) -> anyhow::Result<()> {
        Ok(())
    }
    #[sqlx::test]
    async fn test_get_table_data(db: PgPool) -> anyhow::Result<()> {
        Ok(())
    }
    #[sqlx::test]
    async fn test_delete_tables_without_owner(db: PgPool) -> anyhow::Result<()> {
        Ok(())
    }
}

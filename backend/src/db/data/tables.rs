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
    use itertools::Itertools;
    use sqlx::{PgPool, query_as};
    use crate::{
        db::{self, create_user},
        model::{
            access::AccessRole,
            data::{CreateTable, UpdateTable},
        },
        test_util,
    };

    #[sqlx::test]
    async fn create_table(db: PgPool) -> anyhow::Result<()> {
        let name: String = "blazinglyfast".into();
        let desc: String = "it's just better".into();

        let table = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: name.clone(),
                description: desc.clone(),
            },
        )
        .await?;

        assert_eq!(table.name, name);
        assert_eq!(table.description, desc);

        let dashboard_ref = sqlx::query_as(r#"SELECT * FROM meta_table WHERE table_id = $1"#)
            .bind(table.table_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(table, dashboard_ref);

        Ok(())
    }

    #[sqlx::test]
    async fn update_table(db: PgPool) -> anyhow::Result<()> {
        let name1: String = "blazinglyfast".into();
        let desc1: String = "it's just better".into();

        let table1 = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: name1.clone(),
                description: desc1.clone(),
            },
        )
        .await?;

        assert_eq!(table1.name, name1);
        assert_eq!(table1.description, desc1);

        let name2: String = "betterthanGO".into();
        let desc2: String = "we love ferris".into();

        let table2 = super::update_table(
            &db,
            table1.table_id,
            UpdateTable {
                name: name2.clone(),
                description: desc2.clone(),
            },
        )
        .await?;

        assert_eq!(table2.name, name2);
        assert_eq!(table2.description, desc2);
        assert_eq!(table1.table_id, table2.table_id);

        let dashboard_ref = sqlx::query_as(r#"SELECT * FROM meta_table WHERE table_id = $1"#)
            .bind(table2.table_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(table2, dashboard_ref);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_table(db: PgPool) -> anyhow::Result<()> {
        let name: String = "blazinglyfast".into();
        let desc: String = "it's just better".into();

        let table = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: name.clone(),
                description: desc.clone(),
            },
        )
        .await?;

        super::delete_table(&db, table.table_id).await?;
        let not_exists: bool = sqlx::query_scalar(
            r#"SELECT NOT EXISTS (SELECT 1 FROM meta_table WHERE table_id = $1)"#,
        )
        .bind(table.table_id)
        .fetch_one(&db)
        .await?;

        assert!(not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn get_table_parent(db: PgPool) -> anyhow::Result<()> {
        let parent = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "parent".into(),
                description: "This is the parent".into(),
            },
        )
        .await?;

        let child = super::create_table(
            &db,
            CreateTable {
                parent_id: Some(parent.table_id),
                name: "child".into(),
                description: "This is the child".into(),
            },
        )
        .await?;

        let parent_id = super::get_table_parent_id(&db, child.table_id).await?;
        assert_eq!(parent_id, child.parent_id);

        let parent_id = super::get_table_parent_id(&db, parent.table_id).await?;
        assert_eq!(parent_id, parent.parent_id);
        Ok(())
    }

    #[sqlx::test]
    async fn get_tables(db: PgPool) -> anyhow::Result<()> {
        let user = create_user(&db, "test".into(), "password".into(), false).await?;
        let table1 = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "Table1".into(),
                description: "This is table 1".into(),
            },
        )
        .await?;
        let table2 = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "Table1".into(),
                description: "This is table 2".into(),
            },
        )
        .await?;

        db::create_access(
            &db,
            crate::model::access::Resource::Table,
            table1.table_id,
            user.user_id,
            AccessRole::Owner,
        )
        .await?;

        db::create_access(
            &db,
            crate::model::access::Resource::Table,
            table2.table_id,
            user.user_id,
            AccessRole::Owner,
        )
        .await?;

        let table_list = super::get_tables(&db, user.user_id)
            .await?
            .into_iter()
            .map(|t| t.table)
            .collect_vec();

        test_util::assert_eq_vec(table_list, vec![table1, table2], |t| t.table_id);
        Ok(())
    }

    #[sqlx::test]
    async fn get_table_children(db: PgPool) -> anyhow::Result<()> {
        let parent = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "ParentTable".into(),
                description: "This is the parent table".into(),
            },
        )
        .await?;

        let child = super::create_table(
            &db,
            CreateTable {
                parent_id: Some(parent.table_id),
                name: "ChildTable".into(),
                description: "This is the child table.".into(),
            },
        )
        .await?;

        let children = super::get_table_children(&db, parent.table_id).await?;

        test_util::assert_eq_vec(children, vec![child], |t| t.table_id);

        Ok(())
    }

    #[sqlx::test]
    async fn get_table_data(db: PgPool) -> anyhow::Result<()> {
        let user = create_user(&db, "test".into(), "password".into(), false).await?;
        let table = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "Table1".into(),
                description: "This is table 1".into(),
            },
        )
        .await?;

        db::create_access(
            &db,
            crate::model::access::Resource::Table,
            table.table_id,
            user.user_id,
            AccessRole::Owner,
        )
        .await?;

        let table_data = super::get_table_data(&db, table.table_id).await?;
        assert_eq!(table.table_id, table_data.table.table_id);

        // TODO: Verify fields and entries are also the same

        Ok(())
    }

    #[sqlx::test]
    async fn delete_tables_without_owner(db: PgPool) -> anyhow::Result<()> {
        let user = create_user(&db, "test".into(), "password".into(), false).await?;
        let table1 = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "Table1".to_string(),
                description: "This is table 1.".to_string(),
            },
        )
        .await?;

        let _table2 = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "Table2".into(),
                description: "This is table 2".into(),
            },
        )
        .await?;
        let _table3 = super::create_table(
            &db,
            CreateTable {
                parent_id: None,
                name: "Table3".into(),
                description: "This is table 3".into(),
            },
        )
        .await?;

        db::create_access(
            &db,
            crate::model::access::Resource::Table,
            table1.table_id,
            user.user_id,
            AccessRole::Owner,
        )
        .await?;

        super::delete_tables_without_owner(&db).await?;

        let count_remaining: (i64,) =
            query_as(r#"SELECT COUNT(*) FROM meta_table WHERE table_id = $1"#)
                .bind(table1.table_id)
                .fetch_one(&db)
                .await?;

        assert_eq!(count_remaining.0, 1);

        let count_del: (i64,) = query_as(r#"SELECT COUNT(*) FROM meta_table WHERE table_id != $1"#)
            .bind(table1.table_id)
            .fetch_one(&db)
            .await?;

        assert_eq!(count_del.0, 0);

        Ok(())
    }
}

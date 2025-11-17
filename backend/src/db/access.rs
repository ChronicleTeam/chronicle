use crate::{
    Id,
    model::access::{AccessRole, GetAccess, Resource},
};
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

pub async fn create_access(
    conn: impl Acquire<'_, Database = Postgres>,
    resource: Resource,
    resource_id: Id,
    user_id: Id,
    access_role: AccessRole,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let tablename = resource.access_tablename();
    sqlx::query(&format!(
        r#"
            INSERT INTO {tablename} (user_id, resource_id, access_role)
            VALUES ($1, $2, $3)
        "#
    ))
    .bind(user_id)
    .bind(resource_id)
    .bind(access_role)
    .execute(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn update_many_access(
    conn: impl Acquire<'_, Database = Postgres>,
    resource: Resource,
    resource_id: Id,
    user_access_roles: impl IntoIterator<Item = (Id, AccessRole)>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let tablename = resource.access_tablename();
    QueryBuilder::new(format!(
        r#"
            UPDATE {tablename} AS t
            SET access_role = v.access_role
            FROM (
        "#
    ))
    .push_values(user_access_roles, |mut builder, (user_id, access_role)| {
        builder.push_bind(user_id).push_bind(access_role);
    })
    .push(format!(
        r#"
            ) AS v(user_id, access_role)
            WHERE t.user_id = v.user_id
            AND t.resource_id = 
        "#
    ))
    .push_bind(resource_id)
    .build()
    .execute(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete_many_access(
    conn: impl Acquire<'_, Database = Postgres>,
    resource: Resource,
    resource_id: Id,
    user_ids: impl IntoIterator<Item = Id>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let tablename = resource.access_tablename();
    QueryBuilder::new(format!(r#"DELETE FROM {tablename} WHERE resource_id = "#))
        .push_bind(resource_id)
        .push(format!(" AND user_id IN ("))
        .push_values(user_ids, |mut builder, user_id| {
            builder.push_bind(user_id);
        })
        .push(")")
        .build()
        .execute(tx.as_mut())
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_all_access(
    executor: impl PgExecutor<'_>,
    resource: Resource,
    resource_id: Id,
) -> sqlx::Result<Vec<GetAccess>> {
    let tablename = resource.access_tablename();
    sqlx::query_as(&format!(
        r#"
            SELECT username, access_role
            FROM {tablename} AS a
            JOIN app_user AS u
            ON a.user_id = u.user_id
            WHERE resource_id = $1

        "#
    ))
    .bind(resource_id)
    .fetch_all(executor)
    .await
}

pub async fn get_access_role(
    executor: impl PgExecutor<'_>,
    resource: Resource,
    resource_id: Id,
    user_id: Id,
) -> sqlx::Result<Option<AccessRole>> {
    let tablename = resource.access_tablename();
    sqlx::query_scalar::<_, AccessRole>(&format!(
        r#"
            SELECT access_role
            FROM {tablename}
            WHERE user_id = $1 AND resource_id = $2
        "#
    ))
    .bind(user_id)
    .bind(resource_id)
    .fetch_optional(executor)
    .await
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        db,
        model::{
            access::{AccessRole, GetAccess, Resource},
            data::CreateTable,
            viz::CreateDashboard,
        },
        test_util,
    };
    use itertools::Itertools;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn create_access(db: PgPool) -> anyhow::Result<()> {
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
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let resources = [
            (Resource::Table, table_id),
            (Resource::Dashboard, dashboard_id),
        ];
        let access_roles = [AccessRole::Owner, AccessRole::Editor, AccessRole::Viewer];
        for (idx, ((resource, resource_id), access_role_1)) in resources
            .into_iter()
            .cartesian_product(access_roles)
            .enumerate()
        {
            let user_id = db::create_user(&db, idx.to_string(), "".into(), false)
                .await?
                .user_id;
            let tablename = resource.access_tablename();
            super::create_access(&db, resource, resource_id, user_id, access_role_1).await?;
            let access_role_2 = sqlx::query_scalar(&format!(
                r#"SELECT access_role FROM {tablename} WHERE resource_id = $1 AND user_id = $2"#
            ))
            .bind(resource_id)
            .bind(user_id)
            .fetch_one(&db)
            .await?;
            assert_eq!(access_role_1, access_role_2)
        }
        Ok(())
    }

    #[sqlx::test]
    async fn update_many_access(db: PgPool) -> anyhow::Result<()> {
        let user_id_1 = db::create_user(&db, "gary".into(), "".into(), false)
            .await?
            .user_id;
        let user_id_2 = db::create_user(&db, "mary".into(), "".into(), false)
            .await?
            .user_id;

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
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let resources = [
            (Resource::Table, table_id),
            (Resource::Dashboard, dashboard_id),
        ];
        for (resource, resource_id) in resources {
            let tablename = resource.access_tablename();
            sqlx::query(&format!(
                r#"INSERT INTO {tablename} (resource_id, user_id, access_role) VALUES ($1, $2, $3), ($1, $4, $5)"#
            ))
            .bind(resource_id)
            .bind(user_id_1)
            .bind(AccessRole::Viewer)
            .bind(user_id_2)
            .bind(AccessRole::Viewer)
            .execute(&db)
            .await?;
            let user_access_roles_1 = vec![
                (user_id_1, AccessRole::Editor),
                (user_id_2, AccessRole::Editor),
            ];
            super::update_many_access(&db, resource, resource_id, user_access_roles_1.clone())
                .await?;
            let user_access_roles_2 = sqlx::query_as(&format!(
                r#"SELECT user_id, access_role FROM {tablename} WHERE resource_id = $1"#
            ))
            .bind(resource_id)
            .fetch_all(&db)
            .await?;
            test_util::assert_eq_vec(user_access_roles_1, user_access_roles_2, |(id, _)| *id);
        }
        Ok(())
    }

    #[sqlx::test]
    async fn delete_many_access(db: PgPool) -> anyhow::Result<()> {
        let user_id_1 = db::create_user(&db, "gary".into(), "".into(), false)
            .await?
            .user_id;
        let user_id_2 = db::create_user(&db, "mary".into(), "".into(), false)
            .await?
            .user_id;

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
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let resources = [
            (Resource::Table, table_id),
            (Resource::Dashboard, dashboard_id),
        ];
        for (resource, resource_id) in resources {
            let tablename = resource.access_tablename();
            sqlx::query(&format!(
                r#"INSERT INTO {tablename} (resource_id, user_id, access_role) VALUES ($1, $2, $3), ($1, $4, $5)"#
            ))
            .bind(resource_id)
            .bind(user_id_1)
            .bind(AccessRole::Viewer)
            .bind(user_id_2)
            .bind(AccessRole::Viewer)
            .execute(&db)
            .await?;
            let user_ids_1 = vec![user_id_1, user_id_2];
            super::delete_many_access(&db, resource, resource_id, user_ids_1.clone()).await?;
            let count: i64 = sqlx::query_scalar(&format!(
                r#"SELECT COUNT(user_id) FROM {tablename} WHERE resource_id = $1"#
            ))
            .bind(resource_id)
            .fetch_one(&db)
            .await?;
            assert_eq!(count, 0);
        }
        Ok(())
    }

    #[sqlx::test]
    async fn get_all_access(db: PgPool) -> anyhow::Result<()> {
        let user_1 = db::create_user(&db, "gary".into(), "".into(), false).await?;
        let user_2 = db::create_user(&db, "mary".into(), "".into(), false).await?;

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
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let resources = [
            (Resource::Table, table_id),
            (Resource::Dashboard, dashboard_id),
        ];
        for (resource, resource_id) in resources {
            let tablename = resource.access_tablename();
            sqlx::query(&format!(
                r#"INSERT INTO {tablename} (resource_id, user_id, access_role) VALUES ($1, $2, $3), ($1, $4, $5)"#
            ))
            .bind(resource_id)
            .bind(user_1.user_id)
            .bind(AccessRole::Viewer)
            .bind(user_2.user_id)
            .bind(AccessRole::Owner)
            .execute(&db)
            .await?;
            let get_access_1 = vec![
                GetAccess {
                    username: user_1.username.clone(),
                    access_role: AccessRole::Viewer,
                },
                GetAccess {
                    username: user_2.username.clone(),
                    access_role: AccessRole::Owner,
                },
            ];
            let get_access_2 = super::get_all_access(&db, resource, resource_id).await?;
            test_util::assert_eq_vec(get_access_1, get_access_2, |x| x.username.clone());
        }
        Ok(())
    }

    #[sqlx::test]
    async fn get_access_role(db: PgPool) -> anyhow::Result<()> {
        let user_id = db::create_user(&db, "gary".into(), "".into(), false)
            .await?
            .user_id;

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
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let resources = [
            (Resource::Table, table_id),
            (Resource::Dashboard, dashboard_id),
        ];
        for (resource, resource_id) in resources {
            assert_eq!(
                super::get_access_role(&db, resource, resource_id, user_id).await?,
                None
            );
            let access_role_1 = AccessRole::Editor;
            let tablename = resource.access_tablename();
            sqlx::query(&format!(
                r#"INSERT INTO {tablename} (resource_id, user_id, access_role) VALUES ($1, $2, $3)"#
            ))
            .bind(resource_id)
            .bind(user_id)
            .bind(AccessRole::Editor)
            .execute(&db)
            .await?;
            assert_eq!(
                super::get_access_role(&db, resource, resource_id, user_id).await?,
                Some(access_role_1)
            );
        }
        Ok(())
    }
}

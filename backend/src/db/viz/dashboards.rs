use crate::{
    Id, db,
    model::{
        access::AccessRole,
        viz::{CreateDashboard, Dashboard, GetDashboard, UpdateDashboard},
    },
};
use sqlx::{Acquire, PgExecutor, Postgres};

/// Add a dashboard to this user.
pub async fn create_dashboard(
    conn: impl Acquire<'_, Database = Postgres>,
    CreateDashboard { name, description }: CreateDashboard,
) -> sqlx::Result<Dashboard> {
    let mut tx = conn.begin().await?;

    let dashboard: Dashboard = sqlx::query_as(
        r#"
            INSERT INTO dashboard (name, description)
            VALUES ($1, $2) 
            RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(dashboard)
}

/// Update the dashboard metadata.
pub async fn update_dashboard(
    conn: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
    UpdateDashboard { name, description }: UpdateDashboard,
) -> sqlx::Result<Dashboard> {
    let mut tx = conn.begin().await?;

    let dashboard: Dashboard = sqlx::query_as(
        r#"
            UPDATE dashboard
            SET name = $1, description = $2
            WHERE dashboard_id = $3
            RETURNING
                dashboard_id,
                user_id,
                name,
                description,
                created_at,
                updated_at
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(dashboard_id)
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(dashboard)
}

/// Delete the dashboard along with its charts.
pub async fn delete_dashboard(
    conn: impl Acquire<'_, Database = Postgres>,
    dashboard_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    let chart_ids: Vec<Id> = sqlx::query_scalar(
        r#"
            SELECT chart_id
            FROM chart
            WHERE dashboard_id = $1
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(tx.as_mut())
    .await?;
    for chart_id in chart_ids {
        db::delete_chart(tx.as_mut(), chart_id).await?;
    }

    sqlx::query(
        r#"
            DELETE FROM dashboard
            WHERE dashboard_id = $1
        "#,
    )
    .bind(dashboard_id)
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Get all dashboards belonging to this user.
pub async fn get_dashboards_for_user(
    executor: impl PgExecutor<'_>,
    user_id: Id,
) -> sqlx::Result<Vec<GetDashboard>> {
    sqlx::query_as(
        r#"
            SELECT *
            FROM dashboard AS d
            JOIN dashboard_access AS a
            ON d.dashboard_id = a.resource_id
            WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
}

pub async fn delete_dashboards_without_owner(
    conn: impl Acquire<'_, Database = Postgres>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    let dashboard_ids: Vec<Id> = sqlx::query_scalar(
        r#"
        SELECT dashboard_id
        FROM dashboard AS d
        WHERE NOT EXISTS (
            SELECT 1
            FROM dashboard_access AS a
            WHERE a.resource_id = d.dashboard_id
            AND a.access_role = $1
        )
    "#,
    )
    .bind(AccessRole::Owner)
    .fetch_all(tx.as_mut())
    .await?;

    for dashboard_id in dashboard_ids {
        delete_dashboard(tx.as_mut(), dashboard_id).await?;
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use anyhow::Ok;
    use axum::Json;
    use sqlx::{PgPool, query_as};

    use crate::{
        db::create_user,
        model::{
            access::AccessRole,
            viz::{CreateDashboard, UpdateDashboard},
        },
    };

    #[sqlx::test]
    async fn create_dashboard(db: PgPool) -> anyhow::Result<()> {
        let name: String = "blazinglyfast".into();
        let desc: String = "it's just better".into();

        let dashboard = super::create_dashboard(
            &db,
            CreateDashboard {
                name: name.clone(),
                description: desc.clone(),
            },
        )
        .await?;

        assert_eq!(dashboard.name, name);
        assert_eq!(dashboard.description, desc);

        let dashboard_ref = sqlx::query_as(r#"SELECT * FROM dashboard WHERE dashboard_id = $1"#)
            .bind(dashboard.dashboard_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(dashboard, dashboard_ref);

        Ok(())
    }

    #[sqlx::test]
    async fn update_dashboard(db: PgPool) -> anyhow::Result<()> {
        let name1: String = "blazinglyfast".into();
        let desc1: String = "it's just better".into();

        let dashboard1 = super::create_dashboard(
            &db,
            CreateDashboard {
                name: name1.clone(),
                description: desc1.clone(),
            },
        )
        .await?;

        assert_eq!(dashboard1.name, name1);
        assert_eq!(dashboard1.description, desc1);

        let name2: String = "betterthanGO".into();
        let desc2: String = "we love ferris".into();

        let dashboard2 = super::update_dashboard(
            &db,
            dashboard1.dashboard_id,
            UpdateDashboard {
                name: name2.clone(),
                description: desc2.clone(),
            },
        )
        .await?;

        assert_eq!(dashboard2.name, name2);
        assert_eq!(dashboard2.description, desc2);
        assert_eq!(dashboard1.dashboard_id, dashboard2.dashboard_id);

        let dashboard_ref = sqlx::query_as(r#"SELECT * FROM dashboard WHERE dashboard_id = $1"#)
            .bind(dashboard2.dashboard_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(dashboard2, dashboard_ref);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_dashboard(db: PgPool) -> anyhow::Result<()> {
        let name: String = "blazinglyfast".into();
        let desc: String = "it's just better".into();

        let dashboard = super::create_dashboard(
            &db,
            CreateDashboard {
                name: name.clone(),
                description: desc.clone(),
            },
        )
        .await?;

        super::delete_dashboard(&db, dashboard.dashboard_id).await?;
        let not_exists: bool = sqlx::query_scalar(
            r#"SELECT NOT EXISTS (SELECT 1 FROM dashboard WHERE dashboard_id = $1)"#,
        )
        .bind(dashboard.dashboard_id)
        .fetch_one(&db)
        .await?;

        assert!(not_exists);

        Ok(())
    }

    #[sqlx::test]
    async fn get_dashboards_for_user(db: PgPool) -> anyhow::Result<()> {
        assert!(super::get_dashboards_for_user(&db, 0).await?.is_empty());
        let user = create_user(&db, "test".into(), "password".into(), false).await?;
        let dashboard1 = super::create_dashboard(
            &db,
            CreateDashboard {
                name: "Dashboard1".into(),
                description: "This is dashboard 1".into(),
            },
        )
        .await?;
        let dashboard2 = super::create_dashboard(
            &db,
            CreateDashboard {
                name: "Dashboard2".into(),
                description: "This is dashboard 2".into(),
            },
        )
        .await?;

        let dashboard_list = super::get_dashboards_for_user(&db, user.user_id).await?;
        assert!(dashboard_list.len() == 2);

        assert!(
            dashboard1 == dashboard_list.get(0).unwrap().dashboard
                || dashboard2 == dashboard_list.get(0).unwrap().dashboard
        );
        assert!(
            dashboard1 == dashboard_list.get(1).unwrap().dashboard
                || dashboard2 == dashboard_list.get(1).unwrap().dashboard
        );

        Ok(())
    }

    #[sqlx::test]
    async fn delete_dashboards_without_owner(db: PgPool) -> anyhow::Result<()> {
        let dashboard1 = super::create_dashboard(
            &db,
            CreateDashboard {
                name: "Dashboard1".into(),
                description: "This is dashboard 1".into(),
            },
        )
        .await?;
        let _dashboard2 = super::create_dashboard(
            &db,
            CreateDashboard {
                name: "Dashboard2".into(),
                description: "This is dashboard 2".into(),
            },
        )
        .await?;
        let _dashboard3 = super::create_dashboard(
            &db,
            CreateDashboard {
                name: "Dashboard3".into(),
                description: "This is dashboard 3".into(),
            },
        )
        .await?;
        let _owner_res1: bool = sqlx::query_scalar(
            r#"INSERT INTO dashboard_access (resource_id, access_role) VALUES ($1, $2)"#,
        )
        .bind(dashboard1.dashboard_id)
        .bind(AccessRole::Owner)
        .fetch_one(&db)
        .await?;

        let _res = super::delete_dashboards_without_owner(&db);

        let count_remaining: (i64,) =
            query_as(r#"SELECT COUNT(*) FROM dashboard WHERE dashboard_id = $1"#)
                .bind(dashboard1.dashboard_id)
                .fetch_one(&db)
                .await?;

        assert_eq!(count_remaining.0, 1);

        let count_del: (i64,) =
            query_as(r#"SELECT COUNT(*) FROM dashboard WHERE dashboard_id != $1"#)
                .bind(dashboard1.dashboard_id)
                .fetch_one(&db)
                .await?;

        assert_eq!(count_del.0, 0);

        Ok(())
    }
}

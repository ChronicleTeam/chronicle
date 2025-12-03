use crate::{
    AppState,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult},
    model::{
        access::{AccessRole, AccessRoleCheck, Resource},
        viz::{CreateDashboard, Dashboard, GetDashboard, SelectDashboard, UpdateDashboard},
    },
};
use aide::{
    NoApi,
    axum::{
        ApiRouter,
        routing::{patch_with, post_with},
    },
};
use axum::{
    Json,
    extract::{Path, State},
};
use axum_login::AuthSession;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/dashboards",
        ApiRouter::new()
            .api_route(
                "/",
                post_with(create_dashboard, docs::create_dashboard)
                    .get_with(get_dashboards, docs::get_dashboards),
            )
            .api_route(
                "/{dashboard_id}",
                patch_with(update_dashboard, docs::update_dashboard)
                    .delete_with(delete_dashboard, docs::delete_dashboard),
            ),
    )
}

async fn create_dashboard(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Json(create_dashboard): Json<CreateDashboard>,
) -> ApiResult<Json<Dashboard>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    let dashboard = db::create_dashboard(&db, create_dashboard).await?;
    db::create_access(
        tx.as_mut(),
        Resource::Dashboard,
        dashboard.dashboard_id,
        user_id,
        AccessRole::Owner,
    )
    .await?;

    tx.commit().await?;
    Ok(Json(dashboard))
}

async fn update_dashboard(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectDashboard { dashboard_id }): Path<SelectDashboard>,
    Json(update_dashboard): Json<UpdateDashboard>,
) -> ApiResult<Json<Dashboard>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    let dashboard = db::update_dashboard(&db, dashboard_id, update_dashboard).await?;

    tx.commit().await?;
    Ok(Json(dashboard))
}

async fn delete_dashboard(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectDashboard { dashboard_id }): Path<SelectDashboard>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    tx.commit().await?;
    db::delete_dashboard(&db, dashboard_id).await?;

    Ok(())
}

async fn get_dashboards(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
) -> ApiResult<Json<Vec<GetDashboard>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    let dashboards = db::get_dashboards_for_user(&db, user_id).await?;

    Ok(Json(dashboards))
}

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs {
    use crate::{
        docs::{DASHBOARDS_TAG, TransformOperationExt, template},
        model::{
            access::{AccessRole, Resource},
            viz::Dashboard,
        },
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    const DASHBOARD_OWNER: [(Resource, AccessRole); 1] = [(Resource::Dashboard, AccessRole::Owner)];

    fn dashboards<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, DASHBOARDS_TAG)
    }

    pub fn create_dashboard(op: TransformOperation) -> TransformOperation {
        dashboards::<Json<Dashboard>>(op, "create_dashboard", "Create a blank dashboard.")
    }

    pub fn update_dashboard(op: TransformOperation) -> TransformOperation {
        dashboards::<Json<Dashboard>>(op, "update_dashboard", "Update a dashboard's metadata.")
            .response_description::<404, ()>("Dashboard not found")
            .required_access(DASHBOARD_OWNER)
    }

    pub fn delete_dashboard(op: TransformOperation) -> TransformOperation {
        dashboards::<()>(
            op,
            "delete_dashboard",
            "Delete a dashboard and all of it's charts and axes.",
        )
        .response_description::<404, ()>("Dashboard not found")
        .required_access(DASHBOARD_OWNER)
    }

    pub fn get_dashboards(op: TransformOperation) -> TransformOperation {
        dashboards::<Json<Vec<Dashboard>>>(
            op,
            "get_dashboards",
            "Get all dashboards viewable to the user.",
        )
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        db,
        model::{
            access::{AccessRole, Resource},
            viz::{CreateDashboard, Dashboard, GetDashboard, UpdateDashboard},
        },
        test_util,
    };
    use anyhow::Ok;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn create_dashboard(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let path = "/api/dashboards";

        let create_dashboard = CreateDashboard {
            name: "test".into(),
            description: "description".into(),
        };

        server
            .post(path)
            .json(&create_dashboard)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;

        let response = server.post(path).json(&create_dashboard).await;
        response.assert_status_ok();
        let dashboard_1: Dashboard = response.json();
        assert_eq!(create_dashboard.name, dashboard_1.name);
        assert_eq!(create_dashboard.description, dashboard_1.description);

        let dashboard_2: Dashboard =
            sqlx::query_as(r#"SELECT * FROM dashboard WHERE dashboard_id = $1"#)
                .bind(dashboard_1.dashboard_id)
                .fetch_one(&db)
                .await?;
        assert_eq!(dashboard_1, dashboard_2);

        let access_role = db::get_access_role(
            &db,
            Resource::Dashboard,
            dashboard_1.dashboard_id,
            user.user_id,
        )
        .await?
        .unwrap();
        assert_eq!(access_role, AccessRole::Owner);
        Ok(())
    }

    #[sqlx::test]
    async fn update_dashboard(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "A".into(),
                description: "B".into(),
            },
        )
        .await?
        .dashboard_id;
        let path = format!("/api/dashboards/{dashboard_id}");

        let update_dashboard = UpdateDashboard {
            name: "C".into(),
            description: "D".into(),
        };

        server
            .patch(&path)
            .json(&update_dashboard)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Dashboard,
            dashboard_id,
            user.user_id,
            AccessRole::Owner,
            async || server.patch(&path).json(&update_dashboard).await,
        )
        .await;

        server
            .patch("/api/dashboards/1000")
            .json(&update_dashboard)
            .await
            .assert_status_not_found();

        let update_dashboard = UpdateDashboard {
            name: "E".into(),
            description: "F".into(),
        };
        let response = server.patch(&path).json(&update_dashboard).await;
        response.assert_status_ok();
        let dashboard_1: Dashboard = response.json();
        assert_eq!(update_dashboard.name, dashboard_1.name);
        assert_eq!(update_dashboard.description, dashboard_1.description);

        let dashboard_2: Dashboard =
            sqlx::query_as(r#"SELECT * FROM dashboard WHERE dashboard_id = $1"#)
                .bind(dashboard_1.dashboard_id)
                .fetch_one(&db)
                .await?;
        assert_eq!(dashboard_1, dashboard_2);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_dashboard(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;

        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "A".into(),
                description: "B".into(),
            },
        )
        .await?
        .dashboard_id;
        let path = format!("/api/dashboards/{dashboard_id}");

        server.delete(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Dashboard,
            dashboard_id,
            user.user_id,
            AccessRole::Owner,
            async || {
                server
                    .delete(&format!("/api/dashboards/{dashboard_id}"))
                    .await
            },
        )
        .await;

        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "C".into(),
                description: "D".into(),
            },
        )
        .await
        .unwrap()
        .dashboard_id;
        db::create_access(
            &db,
            Resource::Dashboard,
            dashboard_id,
            user.user_id,
            AccessRole::Owner,
        )
        .await?;
        let path = format!("/api/dashboards/{dashboard_id}");

        server
            .delete("/api/dashboards/1000")
            .await
            .assert_status_not_found();

        server.delete(&path).await.assert_status_ok();

        let not_exists: bool = sqlx::query_scalar(
            r#"SELECT NOT EXISTS (SELECT 1 FROM dashboard WHERE dashboard_id = $1)"#,
        )
        .bind(dashboard_id)
        .fetch_one(&db)
        .await?;
        assert!(not_exists);

        server.delete(&path).await.assert_status_not_found();
        Ok(())
    }

    #[sqlx::test]
    async fn get_dashboards(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let user = db::create_user(&db, "test".into(), "".into(), false).await?;

        let mut dashboards_1 = Vec::new();
        for (idx, access_role) in [AccessRole::Viewer, AccessRole::Editor, AccessRole::Owner]
            .into_iter()
            .enumerate()
        {
            let dashboard = db::create_dashboard(
                &db,
                CreateDashboard {
                    name: idx.to_string(),
                    description: idx.to_string(),
                },
            )
            .await?;
            db::create_access(
                &db,
                Resource::Dashboard,
                dashboard.dashboard_id,
                user.user_id,
                access_role,
            )
            .await?;
            dashboards_1.push(GetDashboard {
                dashboard,
                access_role,
            });
        }

        let path = "/api/dashboards";

        server.get(&path).await.assert_status_unauthorized();

        test_util::login_session(&mut server, &user).await;

        let response = server.get(&path).await;
        response.assert_status_ok();
        let dashboards_2: Vec<GetDashboard> = response.json();
        test_util::assert_eq_vec(dashboards_1, dashboards_2, |d| d.dashboard.dashboard_id);
        Ok(())
    }
}

//! Routes for managing dashboard charts.

use crate::{
    AppState,
    auth::AppAuthSession,
    db,
    error::{ApiError, ApiResult},
    model::{
        access::{AccessRole, AccessRoleCheck, Resource},
        viz::{Chart, ChartData, CreateChart, SelectChart, SelectDashboard, UpdateChart},
    },
};
use aide::{
    NoApi,
    axum::{
        ApiRouter,
        routing::{get_with, patch_with, post_with},
    },
};
use axum::{
    Json,
    extract::{Path, State},
};
use axum_login::AuthSession;

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/dashboards/{dashboard_id}/charts",
        ApiRouter::new()
            .api_route(
                "/",
                post_with(create_chart, docs::create_chart).get_with(get_charts, docs::get_charts),
            )
            .api_route(
                "/{chart_id}",
                patch_with(update_chart, docs::update_chart)
                    .delete_with(delete_chart, docs::delete_chart),
            )
            .api_route(
                "/{chart_id}/data",
                get_with(get_chart_data, docs::get_chart_data),
            ),
    )
}

async fn create_chart(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectDashboard { dashboard_id }): Path<SelectDashboard>,
    Json(create_chart): Json<CreateChart>,
) -> ApiResult<Json<Chart>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Editor)?;
    db::get_access_role(tx.as_mut(), Resource::Table, create_chart.table_id, user_id)
        .await?
        .check(AccessRole::Viewer)?;

    let chart = db::create_chart(tx.as_mut(), dashboard_id, create_chart).await?;

    tx.commit().await?;
    Ok(Json(chart))
}

async fn update_chart(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectChart {
        dashboard_id,
        chart_id,
    }): Path<SelectChart>,
    Json(update_chart): Json<UpdateChart>,
) -> ApiResult<Json<Chart>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Editor)?;
    if !db::chart_exists(tx.as_mut(), dashboard_id, chart_id).await? {
        return Err(ApiError::NotFound);
    };

    let chart = db::update_chart(tx.as_mut(), chart_id, update_chart).await?;

    tx.commit().await?;
    Ok(Json(chart))
}

async fn delete_chart(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectChart {
        dashboard_id,
        chart_id,
    }): Path<SelectChart>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Editor)?;
    if !db::chart_exists(tx.as_mut(), dashboard_id, chart_id).await? {
        return Err(ApiError::NotFound);
    };

    db::delete_chart(tx.as_mut(), chart_id).await?;

    tx.commit().await?;
    Ok(())
}

async fn get_charts(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectDashboard { dashboard_id }): Path<SelectDashboard>,
) -> ApiResult<Json<Vec<Chart>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_access_role(&db, Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Viewer)?;

    let charts = db::get_charts(&db, dashboard_id).await?;

    Ok(Json(charts))
}
async fn get_chart_data(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectChart {
        dashboard_id,
        chart_id,
    }): Path<SelectChart>,
) -> ApiResult<Json<ChartData>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_access_role(&db, Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Viewer)?;
    if !db::chart_exists(&db, dashboard_id, chart_id).await? {
        return Err(ApiError::NotFound);
    };

    let chart_data = db::get_chart_data(&db, chart_id).await?;

    Ok(Json(chart_data))
}

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs {
    use crate::{
        docs::{CHARTS_TAG, TransformOperationExt, template},
        model::{
            access::{AccessRole, Resource},
            viz::{Chart, ChartData},
        },
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    const DASHBOARD_EDITOR_TABLE_VIEWER: [(Resource, AccessRole); 2] = [
        (Resource::Dashboard, AccessRole::Editor),
        (Resource::Table, AccessRole::Viewer),
    ];
    const DASHBOARD_EDITOR: [(Resource, AccessRole); 1] =
        [(Resource::Dashboard, AccessRole::Editor)];
    const DASHBOARD_VIEWER: [(Resource, AccessRole); 1] =
        [(Resource::Dashboard, AccessRole::Viewer)];

    fn charts<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, CHARTS_TAG)
    }

    pub fn create_chart(op: TransformOperation) -> TransformOperation {
        charts::<Json<Chart>>(op, "create_chart", "Create a blank chart.")
            .response_description::<404, ()>("Dashboard not found\n\nTable not found")
            .required_access(DASHBOARD_EDITOR_TABLE_VIEWER)
    }

    pub fn update_chart(op: TransformOperation) -> TransformOperation {
        charts::<Json<Chart>>(op, "update_chart", "Update a chart's metadata.")
            .response_description::<404, ()>("Dashboard not found\n\nChart not found")
            .required_access(DASHBOARD_EDITOR)
    }

    pub fn delete_chart(op: TransformOperation) -> TransformOperation {
        charts::<()>(op, "delete_chart", "Delete a chart and its axes.")
            .response_description::<404, ()>("Dashboard not found\n\nChart not found")
            .required_access(DASHBOARD_EDITOR)
    }

    pub fn get_charts(op: TransformOperation) -> TransformOperation {
        charts::<Json<Vec<Chart>>>(op, "get_charts", "Get all charts for this dashboard.")
            .response_description::<404, ()>("Dashboard not found")
            .required_access(DASHBOARD_VIEWER)
    }

    pub fn get_chart_data(op: TransformOperation) -> TransformOperation {
        charts::<Json<ChartData>>(
            op,
            "get_chart_data",
            "Get the chart's metadata, axes metadata, and data points.
            Used for building and displaying the chart.",
        )
        .response_description::<404, ()>("Dashboard not found\n\nChart not found")
        .required_access(DASHBOARD_VIEWER)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        Id, db,
        model::{
            Cell,
            access::{AccessRole, Resource},
            data::{CreateField, CreateTable, FieldKind, FieldMetadata},
            viz::{
                Aggregate, AxisField, AxisKind, Chart, ChartKind, CreateAxis, CreateChart,
                CreateDashboard, UpdateChart,
            },
        },
        test_util,
    };
    use itertools::Itertools;
    use serde_json::{Value, json};
    use sqlx::PgPool;
    use std::collections::HashMap;

    #[sqlx::test]
    async fn create_chart(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "Test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let path = format!("/api/dashboards/{dashboard_id}/charts");

        let create_chart = CreateChart {
            table_id,
            name: "Test".into(),
            chart_kind: ChartKind::Bar,
        };
        server
            .post(&path)
            .json(&create_chart)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;

        db::create_access(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Owner,
        )
        .await?;
        test_util::test_access_control(
            &db,
            Resource::Dashboard,
            dashboard_id,
            user.user_id,
            AccessRole::Editor,
            async || server.post(&path).json(&create_chart).await,
        )
        .await;

        db::delete_many_access(&db, Resource::Table, table_id, [user.user_id]).await?;
        test_util::test_access_control(
            &db,
            Resource::Table,
            table_id,
            user.user_id,
            AccessRole::Viewer,
            async || server.post(&path).json(&create_chart).await,
        )
        .await;

        server
            .post("/api/dashboards/1000/charts")
            .json(&create_chart)
            .await
            .assert_status_not_found();

        let create_chart = CreateChart {
            table_id,
            name: "abcdef".into(),
            chart_kind: ChartKind::Bar,
        };
        let response = server.post(&path).json(&create_chart).await;
        response.assert_status_ok();
        let chart_1: Chart = response.json();
        assert_eq!(chart_1.table_id, create_chart.table_id);
        assert_eq!(chart_1.name, create_chart.name);
        assert_eq!(chart_1.chart_kind, create_chart.chart_kind);
        let chart_2: Chart = sqlx::query_as(r#"SELECT * FROM chart WHERE chart_id = $1"#)
            .bind(chart_1.chart_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(chart_1, chart_2);
        Ok(())
    }

    #[sqlx::test]
    async fn update_chart(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "Test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let chart_id = db::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id,
                name: "abc".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?
        .chart_id;
        let path = format!("/api/dashboards/{dashboard_id}/charts/{chart_id}");

        let update_chart = UpdateChart {
            name: "def".into(),
            chart_kind: ChartKind::Table,
        };
        server
            .patch(&path)
            .json(&update_chart)
            .await
            .assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Dashboard,
            dashboard_id,
            user.user_id,
            AccessRole::Editor,
            async || server.patch(&path).json(&update_chart).await,
        )
        .await;

        for path_wrong in [
            format!("/api/dashboards/{dashboard_id}/charts/1000"),
            format!("/api/dashboards/1000/charts/{chart_id}"),
        ] {
            server
                .patch(&path_wrong)
                .json(&update_chart)
                .await
                .assert_status_not_found();
        }

        let update_chart = UpdateChart {
            name: "ghj".into(),
            chart_kind: ChartKind::Line,
        };
        let response = server.patch(&path).json(&update_chart).await;
        response.assert_status_ok();
        let chart_1: Chart = response.json();
        assert_eq!(chart_1.name, update_chart.name);
        assert_eq!(chart_1.chart_kind, update_chart.chart_kind);
        let chart_2: Chart = sqlx::query_as(r#"SELECT * FROM chart WHERE chart_id = $1"#)
            .bind(chart_1.chart_id)
            .fetch_one(&db)
            .await?;
        assert_eq!(chart_1, chart_2);
        Ok(())
    }

    #[sqlx::test]
    async fn delete_chart(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "Test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;
        let chart_id = db::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id,
                name: "abc".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?
        .chart_id;
        let path = format!("/api/dashboards/{dashboard_id}/charts/{chart_id}");

        server.delete(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Dashboard,
            dashboard_id,
            user.user_id,
            AccessRole::Editor,
            async || {
                let chart_id = db::create_chart(
                    &db,
                    dashboard_id,
                    CreateChart {
                        table_id,
                        name: "abc".into(),
                        chart_kind: ChartKind::Bar,
                    },
                )
                .await
                .unwrap()
                .chart_id;
                server
                    .delete(&format!("/api/dashboards/{dashboard_id}/charts/{chart_id}"))
                    .await
            },
        )
        .await;

        for path_wrong in [
            format!("/api/dashboards/{dashboard_id}/charts/1000"),
            format!("/api/dashboards/1000/charts/{chart_id}"),
        ] {
            server.delete(&path_wrong).await.assert_status_not_found();
        }

        server.delete(&path).await.assert_status_ok();
        let not_exists: bool =
            sqlx::query_scalar(r#"SELECT NOT EXISTS (SELECT 1 FROM chart WHERE chart_id = $1)"#)
                .bind(chart_id)
                .fetch_one(&db)
                .await?;
        assert!(not_exists);

        server.delete(&path).await.assert_status_not_found();
        Ok(())
    }

    #[sqlx::test]
    async fn get_charts(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "Test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let mut charts_1 = Vec::new();
        for (idx, chart_kind) in [ChartKind::Bar, ChartKind::Table, ChartKind::Line]
            .into_iter()
            .enumerate()
        {
            let name = idx.to_string();
            let table_id = db::create_table(
                &db,
                CreateTable {
                    name: name.clone(),
                    description: "".into(),
                    parent_id: None,
                },
            )
            .await?
            .table_id;
            charts_1.push(
                db::create_chart(
                    &db,
                    dashboard_id,
                    CreateChart {
                        table_id,
                        name,
                        chart_kind,
                    },
                )
                .await?,
            );
        }
        let path = format!("/api/dashboards/{dashboard_id}/charts");

        server.get(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Dashboard,
            dashboard_id,
            user.user_id,
            AccessRole::Viewer,
            async || server.get(&path).await,
        )
        .await;

        server
            .get("/api/dashboards/1000/charts")
            .await
            .assert_status_not_found();

        let response = server.get(&path).await;
        response.assert_status_ok();
        let charts_2 = response.json();
        test_util::assert_eq_vec(charts_1, charts_2, |c| c.chart_id);
        Ok(())
    }

    #[sqlx::test]
    async fn get_chart_data(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let dashboard_id = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "Test".into(),
                description: "".into(),
            },
        )
        .await?
        .dashboard_id;
        let table_id = db::create_table(
            &db,
            CreateTable {
                name: "Test".into(),
                description: "".into(),
                parent_id: None,
            },
        )
        .await?
        .table_id;

        let field = db::create_field(
            &db,
            table_id,
            CreateField {
                name: "Test".into(),
                field_kind: FieldKind::Checkbox,
            },
        )
        .await?;
        let _entries = db::create_entries(
            &db,
            table_id,
            None,
            vec![FieldMetadata::from_field(field.clone())],
            vec![
                vec![Cell::Boolean(false)],
                vec![Cell::Boolean(true)],
                vec![Cell::Boolean(true)],
                vec![Cell::Boolean(false)],
                vec![Cell::Boolean(true)],
                vec![Cell::Boolean(false)],
                vec![Cell::Boolean(false)],
            ],
        )
        .await?;
        let chart_1 = db::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id,
                name: "Test".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?;
        let axes = db::set_axes(
            &db,
            chart_1.chart_id,
            table_id,
            vec![
                CreateAxis {
                    field_id: field.field_id,
                    axis_kind: AxisKind::X,
                    aggregate: None,
                },
                CreateAxis {
                    field_id: field.field_id,
                    axis_kind: AxisKind::X,
                    aggregate: Some(Aggregate::Count),
                },
            ],
        )
        .await?;
        let (group_id, count_id) = axes
            .iter()
            .sorted_by_key(|a| a.aggregate.is_some())
            .map(|a| a.axis_id)
            .collect_tuple()
            .unwrap();
        let path = format!(
            "/api/dashboards/{dashboard_id}/charts/{}/data",
            chart_1.chart_id
        );

        server.get(&path).await.assert_status_unauthorized();

        let user = db::create_user(&db, "test".into(), "".into(), false).await?;
        test_util::login_session(&mut server, &user).await;
        test_util::test_access_control(
            &db,
            Resource::Dashboard,
            dashboard_id,
            user.user_id,
            AccessRole::Viewer,
            async || server.get(&path).await,
        )
        .await;

        for path_wrong in [
            format!("/api/dashboards/1000/charts/{}/data", chart_1.chart_id),
            format!("/api/dashboards/{dashboard_id}/charts/1000/data"),
        ] {
            server.get(&path_wrong).await.assert_status_not_found();
        }

        let response = server.get(&path).await;
        response.assert_status_ok();
        let mut chart_data: Value = response.json();

        let chart_2: Chart =
            serde_json::from_value(chart_data.get_mut("chart").unwrap().take()).unwrap();
        assert_eq!(chart_1, chart_2);

        let axes_fields_1 = axes
            .into_iter()
            .map(|axis| AxisField {
                axis,
                field_name: field.name.clone(),
                field_kind: field.field_kind.clone(),
            })
            .collect_vec();
        let axes_fields_2: Vec<AxisField> =
            serde_json::from_value(chart_data.get_mut("axes").unwrap().take()).unwrap();
        test_util::assert_eq_vec(axes_fields_1, axes_fields_2, |a| a.axis.axis_id);

        let cells_1 = vec![
            HashMap::from([(group_id, json!(false)), (count_id, json!(4))]),
            HashMap::from([(group_id, json!(true)), (count_id, json!(3))]),
        ];
        let cells_2: Vec<HashMap<Id, Value>> =
            serde_json::from_value(chart_data.get_mut("cells").unwrap().take()).unwrap();
        test_util::assert_eq_vec(cells_1, cells_2, |row| row[&group_id].as_bool().unwrap());
        Ok(())
    }
}

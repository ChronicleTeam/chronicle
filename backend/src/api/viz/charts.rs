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

/// Create a blank chart.
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard or table
/// - [ApiError::NotFound]: Dashboard or table not found
///
async fn create_chart(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectDashboard { dashboard_id }): Path<SelectDashboard>,
    Json(create_chart): Json<CreateChart>,
) -> ApiResult<Json<Chart>> {
    let user_id = user.ok_or(ApiError::Forbidden)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Editor)?;
    db::get_access(tx.as_mut(), Resource::Table, create_chart.table_id, user_id)
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
    let user_id = user.ok_or(ApiError::Forbidden)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
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
    let user_id = user.ok_or(ApiError::Forbidden)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
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

    db::get_access(&db, Resource::Dashboard, dashboard_id, user_id)
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
    let user_id = user.ok_or(ApiError::Forbidden)?.user_id;

    db::get_access(&db, Resource::Dashboard, dashboard_id, user_id)
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
            access::AccessRole,
            viz::{Chart, ChartData},
        },
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    const DASHBOARD_EDITOR_TABLE_VIEWER: [(&str, AccessRole); 2] = [
        ("Dashboard", AccessRole::Editor),
        ("Table", AccessRole::Viewer),
    ];
    const DASHBOARD_EDITOR: [(&str, AccessRole); 1] = [("Dashboard", AccessRole::Editor)];
    const DASHBOARD_VIEWER: [(&str, AccessRole); 1] = [("Dashboard", AccessRole::Viewer)];

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

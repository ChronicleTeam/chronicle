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
        model::{access::AccessRole, viz::Dashboard},
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    const DASHBOARD_OWNER: [(&str, AccessRole); 1] = [("Dashboard", AccessRole::Owner)];

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
    use aide::{OperationOutput, transform::TransformOperation};
    use anyhow::Ok;
    use sqlx::PgPool;

    use crate::{db, model::viz::CreateDashboard, test_util};

    #[sqlx::test]
    async fn create_dashboard(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        let path = "/api/";

        Ok(())
    }

    #[sqlx::test]
    async fn update_dashboard(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        Ok(())
    }

    #[sqlx::test]
    async fn delete_dashboard(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;
        Ok(())
    }

    #[sqlx::test]
    async fn get_dashboards(db: PgPool) -> anyhow::Result<()> {
        let mut server = test_util::server(db.clone()).await;

        let db1 = db::create_dashboard(
            &db,
            CreateDashboard {
                name: "blazinglyfast".into(),
                description: "it's just better".into(),
            },
        );

        Ok(())
    }
}

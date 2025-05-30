use crate::{
    db::{self, AuthSession}, error::{ApiError, ApiResult}, model::{
        data::FieldKind,
        viz::{Aggregate, Axis, SetAxes},
    }, routes::ApiState, Id
};
use axum::{
    extract::{Path, State},
    routing::put,
    Json, Router,
};
use itertools::Itertools;
use std::collections::HashMap;

const FIELD_NOT_FOUND: &str = "Field not found";
const INVALID_AXIS_AGGREGATE: &str = "Axis aggregate is invalid for this field";

pub fn router() -> Router<ApiState> {
    Router::new().nest(
        "/dashboards/{dashboard-id}/charts/{chart-id}/axes",
        Router::new().route("/", put(set_axes)),
    )
}

/// Set all the axes of the specified chart.
/// 
/// This is the only way to modify chart axes because the dynamic view needs to
/// be rebuilt and it is much more convienient when receiving all the axes at once.
/// 
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User does not have access to this dashboard or chart
/// - [ApiError::NotFound]: Dashboard or chart not found
/// - [ApiError::UnprocessableEntity]:
///     - <field_id>: [FIELD_NOT_FOUND]
///     - <field_id>: [INVALID_AXIS_AGGREGATE]
/// 
async fn set_axes(
    AuthSession { user, .. }: AuthSession,
    State(ApiState { pool, .. }): State<ApiState>,
    Path((dashboard_id, chart_id)): Path<(Id, Id)>,
    Json(SetAxes(axes)): Json<SetAxes>,
) -> ApiResult<Json<Vec<Axis>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;
    db::check_chart_relation(&pool, dashboard_id, chart_id)
        .await?
        .to_api_result()?;

    let table_id = db::get_chart_table_id(&pool, chart_id).await?;


    let field_kinds: HashMap<_, _> = db::get_fields_metadata(&pool, table_id)
        .await?
        .into_iter()
        .map(|field| (field.field_id, field.field_kind.0))
        .collect();

    let axes = axes
        .into_iter()
        .map(|axis| {
            let field_kind =
                &field_kinds
                    .get(&axis.field_id)
                    .ok_or(ApiError::unprocessable_entity([(
                        axis.field_id.to_string(),
                        FIELD_NOT_FOUND,
                    )]))?;

            if let Some(aggregate) = &axis.aggregate {
                validate_axis(&aggregate, field_kind).map_err(|message| {
                    ApiError::unprocessable_entity([(axis.field_id.to_string(), message)])
                })?;
            }

            ApiResult::Ok(axis)
        })
        .try_collect()?;

    let axes = db::set_axes(&pool, chart_id, table_id, &field_kinds, axes).await?;

    Ok(Json(axes))
}

/// Validate that the axis aggregate and field_kind are compatible
fn validate_axis(aggregate: &Aggregate, field_kind: &FieldKind) -> Result<(), &'static str> {
    match (aggregate, field_kind) {
        (Aggregate::Count, _)
        | (
            Aggregate::Sum,
            FieldKind::Integer { .. } | FieldKind::Float { .. } | FieldKind::Money { .. },
        )
        | (
            Aggregate::Average,
            FieldKind::Integer { .. }
            | FieldKind::Float { .. }
            | FieldKind::Money { .. }
            | FieldKind::Progress { .. },
        )
        | (
            Aggregate::Min | Aggregate::Max,
            FieldKind::Text { .. }
            | FieldKind::Integer { .. }
            | FieldKind::Float { .. }
            | FieldKind::Money { .. }
            | FieldKind::Progress { .. }
            | FieldKind::DateTime { .. },
        ) => Ok(()),
        _ => Err(INVALID_AXIS_AGGREGATE),
    }
}

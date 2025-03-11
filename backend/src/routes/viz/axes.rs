use std::collections::HashMap;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage},
    model::{
        data::FieldKind,
        viz::{Aggregate, Axis, SetAxes},
    },
    routes::ApiState,
    Id,
};
use axum::{
    extract::{Path, State},
    routing::put,
    Json, Router,
};
use itertools::Itertools;

const FIELD_NOT_FOUND: ErrorMessage = ErrorMessage::new_static("field_id", "Field not found");

const INVALID_AXIS_AGGREGATE: &str = "Axis aggregate is invalid for this field";

pub fn router() -> Router<ApiState> {
    Router::new().route(
        "/dashboards/{dashboard-id}/charts/{chart-id}",
        put(set_axes),
    )
}

async fn set_axes(
    State(ApiState { pool, .. }): State<ApiState>,
    Path((dashboard_id, chart_id)): Path<(Id, Id)>,
    Json(set_axes): Json<SetAxes>,
) -> ApiResult<Json<Vec<Axis>>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_dashboard_relation(&pool, user_id, dashboard_id)
        .await?
        .to_api_result()?;

    db::check_chart_relation(&pool, dashboard_id, chart_id)
        .await?
        .to_api_result()?;

    // let chart = db::get_chart(&pool, )

    let fields: HashMap<_, _> = db::get_fields(&pool, set_axes.table_id)
        .await?
        .into_iter()
        .map(|field| (field.field_id, field))
        .collect();

    let axes_and_fields = set_axes.axes.into_iter().map(|axis| {
        let field = fields
            .remove(&axis.field_id)
            .ok_or(ApiError::unprocessable_entity([FIELD_NOT_FOUND]))?;

        if let Some(aggregate) = &axis.aggregate {
            validate_axis(
                &aggregate,
                &field.field_kind,
            )
            .map_err(|message| {
                ApiError::unprocessable_entity([ErrorMessage::new(
                    axis.field_id.to_string(),
                    message,
                )])
            })?;
        }
        
        Ok((axis, field))
    
    }).try_collect()?;

    let axes = db::set_axes(&pool, chart, table, axes_and_fields).await?;

    Ok(Json(axes))
}

fn validate_axis(aggregate: &Aggregate, field_kind: &FieldKind) -> Result<(), &'static str> {
    match (aggregate, field_kind) {
        (
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

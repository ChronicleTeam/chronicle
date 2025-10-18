use crate::{
    AppState,
    auth::AppAuthSession,
    db::{self},
    error::{ApiError, ApiResult},
    model::{
        data::FieldKind,
        users::{AccessRole, AccessRoleCheck},
        viz::{Aggregate, Axis, SelectChart, SetAxes},
    },
};
use aide::{
    NoApi,
    axum::{ApiRouter, routing::put_with},
};
use axum::{
    Json,
    extract::{Path, State},
};
use axum_login::AuthSession;
use itertools::Itertools;
use std::collections::HashMap;

const FIELD_NOT_FOUND: &str = "Field not found";
const INVALID_AXIS_AGGREGATE: &str = "Axis aggregate is invalid for this field";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/dashboards/{dashboard_id}/charts/{chart_id}/axes",
        ApiRouter::new().api_route("/", put_with(set_axes, docs::set_axes)),
    )
}

async fn set_axes(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectChart {
        dashboard_id,
        chart_id,
    }): Path<SelectChart>,
    Json(SetAxes(axes)): Json<SetAxes>,
) -> ApiResult<Json<Vec<Axis>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_dashboard_access(&db, user_id, dashboard_id)
        .await?
        .check(AccessRole::Editor)?;
    if !db::chart_exists(&db, dashboard_id, chart_id).await? {
        return Err(ApiError::NotFound);
    };

    let table_id = db::get_chart_table_id(&db, chart_id).await?;

    let field_kinds: HashMap<_, _> = db::get_fields_metadata(&db, table_id)
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
                    .ok_or(ApiError::UnprocessableEntity(format!(
                        "{}: {FIELD_NOT_FOUND}",
                        axis.field_id,
                    )))?;

            if let Some(aggregate) = &axis.aggregate {
                validate_axis(&aggregate, field_kind).map_err(|message| {
                    ApiError::UnprocessableEntity(format!("{}: {message}", axis.field_id,))
                })?;
            }

            ApiResult::Ok(axis)
        })
        .try_collect()?;

    let axes = db::set_axes(&db, chart_id, table_id, &field_kinds, axes).await?;

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

mod docs {
    use crate::{
        api::viz::axes::{FIELD_NOT_FOUND, INVALID_AXIS_AGGREGATE},
        docs::{AXES_TAG, TransformOperationExt, template},
        model::{users::AccessRole, viz::Axis},
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;
    use itertools::Itertools;

    const DASHBOARD_EDITOR: [(&str, AccessRole); 1] = [("Dashboard", AccessRole::Editor)];

    fn axes<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, AXES_TAG)
    }

    pub fn set_axes(op: TransformOperation) -> TransformOperation {
        let errors = [FIELD_NOT_FOUND, INVALID_AXIS_AGGREGATE]
            .into_iter()
            .map(|v| format!("<field_id> : {v}"))
            .join("\n\n");

        axes::<Json<Vec<Axis>>>(
            op,
            "set_axes",
            "Set all the axes of the specified chart and rebuild the dynamic view.",
        )
        .response_description::<404, ()>("Dashboard not found\n\nChart not found")
        .response_description::<422, String>(&errors)
        .required_access(DASHBOARD_EDITOR)
    }
}

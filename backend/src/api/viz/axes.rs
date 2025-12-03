use crate::{
    AppState,
    api::NO_DATA_IN_REQUEST_BODY,
    auth::AppAuthSession,
    db::{self},
    error::{ApiError, ApiResult},
    model::{
        access::{AccessRole, AccessRoleCheck, Resource},
        data::FieldKind,
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
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Dashboard, dashboard_id, user_id)
        .await?
        .check(AccessRole::Editor)?;
    if !db::chart_exists(tx.as_mut(), dashboard_id, chart_id).await? {
        return Err(ApiError::NotFound);
    };

    if axes.is_empty() {
        return Err(ApiError::BadRequest(NO_DATA_IN_REQUEST_BODY.into()));
    }

    let table_id = db::get_chart_table_id(tx.as_mut(), chart_id).await?;

    let field_kinds: HashMap<_, _> = db::get_fields_metadata(tx.as_mut(), table_id)
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
                validate_axis(aggregate, field_kind).map_err(|message| {
                    ApiError::UnprocessableEntity(format!("{}: {message}", axis.field_id,))
                })?;
            }
            ApiResult::Ok(axis)
        })
        .try_collect()?;

    let axes = db::set_axes(tx.as_mut(), chart_id, table_id, axes).await?;

    tx.commit().await?;
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

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs {
    use crate::{
        api::{
            NO_DATA_IN_REQUEST_BODY,
            viz::axes::{FIELD_NOT_FOUND, INVALID_AXIS_AGGREGATE},
        },
        docs::{AXES_TAG, TransformOperationExt, template},
        model::{
            access::{AccessRole, Resource},
            viz::Axis,
        },
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;
    use itertools::Itertools;

    const DASHBOARD_EDITOR: [(Resource, AccessRole); 1] =
        [(Resource::Dashboard, AccessRole::Editor)];

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
        .response_description::<40, String>(NO_DATA_IN_REQUEST_BODY)
        .response_description::<404, ()>("Dashboard not found\n\nChart not found")
        .response_description::<422, String>(&errors)
        .required_access(DASHBOARD_EDITOR)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::collections::HashMap;

    use itertools::Itertools;
    use sqlx::PgPool;

    use crate::{
        db,
        model::{
            Cell,
            access::{AccessRole, Resource},
            data::{CreateField, CreateTable, FieldKind, FieldMetadata},
            viz::{
                Aggregate, Axis, AxisKind, ChartKind, CreateAxis, CreateChart, CreateDashboard,
                SetAxes,
            },
        },
        test_util,
    };

    #[sqlx::test]
    async fn set_axes(db: PgPool) -> anyhow::Result<()> {
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

        let text_field = FieldMetadata::from_field(
            db::create_field(
                &db,
                table_id,
                CreateField {
                    name: "Text".into(),
                    field_kind: FieldKind::Text { is_required: true },
                },
            )
            .await?,
        );

        let integer_field = FieldMetadata::from_field(
            db::create_field(
                &db,
                table_id,
                CreateField {
                    name: "Integer".into(),
                    field_kind: FieldKind::Integer {
                        is_required: true,
                        range_start: None,
                        range_end: None,
                    },
                },
            )
            .await?,
        );
        let _entries = db::create_entries(
            &db,
            table_id,
            None,
            vec![text_field.clone(), integer_field.clone()],
            vec![
                vec![Cell::String("A".into()), Cell::Integer(1)],
                vec![Cell::String("A".into()), Cell::Integer(2)],
                vec![Cell::String("B".into()), Cell::Integer(3)],
                vec![Cell::String("B".into()), Cell::Integer(4)],
                vec![Cell::String("B".into()), Cell::Integer(5)],
                vec![Cell::String("C".into()), Cell::Integer(6)],
                vec![Cell::String("C".into()), Cell::Integer(7)],
            ],
        )
        .await?;
        let chart_id = db::create_chart(
            &db,
            dashboard_id,
            CreateChart {
                table_id,
                name: "Test".into(),
                chart_kind: ChartKind::Bar,
            },
        )
        .await?
        .chart_id;

        let path = format!("/api/dashboards/{dashboard_id}/charts/{chart_id}/axes");

        let set_axes = SetAxes(vec![CreateAxis {
            field_id: text_field.field_id,
            axis_kind: AxisKind::X,
            aggregate: None,
        }]);

        server
            .put(&path)
            .json(&set_axes)
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
            async || server.put(&path).json(&set_axes).await,
        )
        .await;

        for path_wrong in [
            format!("/api/dashboards/1000/charts/{chart_id}/axes"),
            format!("/api/dashboards/{dashboard_id}/charts/1000/axes"),
        ] {
            server
                .put(&path_wrong)
                .json(&set_axes)
                .await
                .assert_status_not_found();
        }

        let create_group_axis = CreateAxis {
            field_id: text_field.field_id,
            axis_kind: AxisKind::X,
            aggregate: None,
        };
        let create_max_axis = CreateAxis {
            field_id: integer_field.field_id,
            axis_kind: AxisKind::Y,
            aggregate: Some(Aggregate::Max),
        };
        let set_axes = SetAxes(vec![create_group_axis.clone(), create_max_axis.clone()]);

        let response = server.put(&path).json(&set_axes).await;
        response.assert_status_ok();
        let axes_1: Vec<Axis> = response.json();
        let (group_axis_1, max_axis_1) = axes_1
            .iter()
            .sorted_by_key(|a| a.aggregate.is_some())
            .collect_tuple()
            .unwrap();
        for (create_axis, axis_1) in [
            (create_group_axis, group_axis_1),
            (create_max_axis, max_axis_1),
        ] {
            assert_eq!(create_axis.field_id, axis_1.field_id);
            assert_eq!(create_axis.axis_kind, axis_1.axis_kind);
            assert_eq!(create_axis.aggregate, axis_1.aggregate);
        }
        let axes_2: Vec<Axis> = sqlx::query_as(r#"SELECT * FROM axis WHERE chart_id = $1"#)
            .bind(chart_id)
            .fetch_all(&db)
            .await?;
        test_util::assert_eq_vec(axes_1, axes_2, |a| a.axis_id);

        let empty_payload = SetAxes(Vec::new());
        server
            .put(&path)
            .json(&empty_payload)
            .await
            .assert_status_bad_request();

        let wrong_field_id = SetAxes(vec![CreateAxis {
            field_id: 1000,
            axis_kind: AxisKind::X,
            aggregate: None,
        }]);
        server
            .put(&path)
            .json(&wrong_field_id)
            .await
            .assert_status_unprocessable_entity();

        let invalid_aggregate = SetAxes(vec![CreateAxis {
            field_id: text_field.field_id,
            axis_kind: AxisKind::X,
            aggregate: Some(Aggregate::Average),
        }]);
        server
            .put(&path)
            .json(&invalid_aggregate)
            .await
            .assert_status_unprocessable_entity();

        Ok(())
    }

    #[test]
    fn validate_axis() {
        let text = FieldKind::Text { is_required: true };
        let integer = FieldKind::Integer {
            is_required: true,
            range_start: None,
            range_end: None,
        };
        let float = FieldKind::Float {
            is_required: true,
            range_start: None,
            range_end: None,
        };
        let money = FieldKind::Money {
            is_required: true,
            range_start: None,
            range_end: None,
        };
        let date_time = FieldKind::DateTime {
            is_required: true,
            range_start: None,
            range_end: None,
        };
        let progress = FieldKind::Progress { total_steps: 100 };
        let web_link = FieldKind::WebLink { is_required: true };
        let checkbox = FieldKind::Checkbox;
        let enumeration = FieldKind::Enumeration {
            is_required: true,
            values: HashMap::new(),
            default_value: 0,
        };
        for (aggregate, field_kinds, is_ok) in [
            (
                Aggregate::Count,
                [
                    &text,
                    &integer,
                    &float,
                    &money,
                    &date_time,
                    &progress,
                    &web_link,
                    &checkbox,
                    &enumeration,
                ]
                .iter(),
                true,
            ),
            (Aggregate::Sum, [&integer, &float, &money].iter(), true),
            (
                Aggregate::Sum,
                [
                    &text,
                    &date_time,
                    &progress,
                    &web_link,
                    &checkbox,
                    &enumeration,
                ]
                .iter(),
                false,
            ),
            (
                Aggregate::Average,
                [&integer, &float, &money, &progress].iter(),
                true,
            ),
            (
                Aggregate::Average,
                [&text, &date_time, &web_link, &checkbox, &enumeration].iter(),
                false,
            ),
            (
                Aggregate::Min,
                [&text, &integer, &float, &money, &progress, &date_time].iter(),
                true,
            ),
            (
                Aggregate::Min,
                [&web_link, &checkbox, &enumeration].iter(),
                false,
            ),
            (
                Aggregate::Max,
                [&text, &integer, &float, &money, &progress, &date_time].iter(),
                true,
            ),
            (
                Aggregate::Max,
                [&web_link, &checkbox, &enumeration].iter(),
                false,
            ),
        ] {
            for field_kind in field_kinds {
                assert_eq!(super::validate_axis(&aggregate, *field_kind).is_ok(), is_ok);
            }
        }
    }
}

use crate::{
    api::NO_DATA_IN_REQUEST_BODY, auth::AppAuthSession, db, error::{ApiError, ApiResult}, model::{
        access::{AccessRole, AccessRoleCheck, Resource},
        data::{
            CreateField, Field, FieldKind, SelectField, SelectTable, SetFieldOrder, UpdateField,
        },
    }, AppState
};
use aide::{
    NoApi,
    axum::{
        ApiRouter,
        routing::{patch_with, post_with},
    },
};
use anyhow::anyhow;
use axum::{
    Json,
    extract::{Path, State},
};
use axum_login::AuthSession;
use itertools::Itertools;
use std::collections::HashSet;

const INVALID_RANGE: &str = "Range start bound is greater than end bound";
const FIELD_ID_NOT_FOUND: &str = "Field ID not found";
const FIELD_ID_MISSING: &str = "Field ID missing";
const INVALID_ORDERING: &str = "Ordering number does not follow the sequence";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest(
        "/tables/{table_id}/fields",
        ApiRouter::new()
            .api_route(
                "/",
                post_with(create_field, docs::create_field).get_with(get_fields, docs::get_fields),
            )
            .api_route(
                "/{field_id}",
                patch_with(update_field, docs::update_field)
                    .delete_with(delete_field, docs::delete_field),
            )
            .api_route("/order", patch_with(set_field_order, docs::set_field_order)),
    )
}

async fn create_field(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    Json(mut create_field): Json<CreateField>,
) -> ApiResult<Json<Field>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    validate_field_kind(&mut create_field.field_kind)?;

    let field = db::create_field(tx.as_mut(), table_id, create_field).await?;

    tx.commit().await?;
    Ok(Json(field))
}

async fn update_field(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectField { table_id, field_id }): Path<SelectField>,
    Json(mut update_field): Json<UpdateField>,
) -> ApiResult<Json<Field>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    if !db::field_exists(tx.as_mut(), table_id, field_id).await? {
        return Err(ApiError::NotFound);
    };

    validate_field_kind(&mut update_field.field_kind)?;

    let field = db::update_field(tx.as_mut(), field_id, update_field).await?;

    tx.commit().await?;
    Ok(Json(field))
}

async fn delete_field(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectField { table_id, field_id }): Path<SelectField>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    if !db::field_exists(tx.as_mut(), table_id, field_id).await? {
        return Err(ApiError::NotFound);
    };

    db::delete_field(tx.as_mut(), field_id).await?;

    tx.commit().await?;
    Ok(())
}

async fn get_fields(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
) -> ApiResult<Json<Vec<Field>>> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;

    db::get_access_role(&db, Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Viewer)?;

    let fields = db::get_fields(&db, table_id).await?;

    Ok(Json(fields))
}

async fn set_field_order(
    NoApi(AuthSession { user, .. }): AppAuthSession,
    State(AppState { db, .. }): State<AppState>,
    Path(SelectTable { table_id }): Path<SelectTable>,
    Json(SetFieldOrder(order)): Json<SetFieldOrder>,
) -> ApiResult<()> {
    let user_id = user.ok_or(ApiError::Unauthorized)?.user_id;
    let mut tx = db.begin().await?;

    db::get_access_role(tx.as_mut(), Resource::Table, table_id, user_id)
        .await?
        .check(AccessRole::Owner)?;

    if order.is_empty() {
        return Err(ApiError::BadRequest(NO_DATA_IN_REQUEST_BODY.into()))
    }

    let mut field_ids: HashSet<_> = db::get_field_ids(tx.as_mut(), table_id)
        .await?
        .into_iter()
        .collect();

    let mut error_messages = order
        .iter()
        .sorted_by_key(|(_, ordering)| **ordering)
        .enumerate()
        .filter_map(|(idx, (field_id, ordering))| {
            if !field_ids.remove(field_id) {
                Some(format!("{field_id}: {FIELD_ID_NOT_FOUND}"))
            } else if idx as i32 != *ordering {
                Some(format!("{field_id}: {INVALID_ORDERING}"))
            } else {
                None
            }
        })
        .collect_vec();
    error_messages.extend(
        field_ids
            .into_iter()
            .map(|field_id| format!("{field_id}: {FIELD_ID_MISSING}")),
    );

    if !error_messages.is_empty() {
        return Err(ApiError::UnprocessableEntity(error_messages.join(", ")));
    }

    db::set_field_order(tx.as_mut(), order).await?;

    tx.commit().await?;
    Ok(())
}

/// Validates a request [FieldKind].
fn validate_field_kind(field_kind: &mut FieldKind) -> ApiResult<()> {
    match field_kind {
        FieldKind::Integer {
            range_start,
            range_end,
            ..
        } => validate_range(*range_start, *range_end)?,
        FieldKind::Float {
            range_start,
            range_end,
            ..
        } => {
            validate_range(*range_start, *range_end)?;
        }
        FieldKind::Money {
            range_start,
            range_end,
            ..
        } => validate_range(*range_start, *range_end)?,
        FieldKind::Progress { total_steps } => {
            *total_steps = (*total_steps).max(1);
        }
        FieldKind::DateTime {
            range_start,
            range_end,
            // date_time_format,
            ..
        } => validate_range(*range_start, *range_end)?,
        FieldKind::Enumeration {
            values,
            default_value,
            ..
        } => {
            if !values.contains_key(&default_value) {
                return Err(
                    anyhow!("enumeration field default value does not map to a value").into(),
                );
            }
        }
        _ => {}
    };
    Ok(())
}

/// Validates the range definition of a field.
fn validate_range<T>(range_start: Option<T>, range_end: Option<T>) -> ApiResult<()>
where
    T: PartialOrd,
{
    if range_start
        .zip(range_end)
        .map_or(true, |(start, end)| start <= end)
    {
        Ok(())
    } else {
        Err(ApiError::UnprocessableEntity(INVALID_RANGE.into()))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
mod docs {
    use crate::{
        api::{data::fields::{FIELD_ID_NOT_FOUND, INVALID_ORDERING, INVALID_RANGE}, NO_DATA_IN_REQUEST_BODY},
        docs::{template, TransformOperationExt, FIELDS_TAG},
        model::{access::AccessRole, data::Field},
    };
    use aide::{OperationOutput, transform::TransformOperation};
    use axum::Json;

    const TABLE_OWNER: [(&str, AccessRole); 1] = [("Table", AccessRole::Owner)];
    const TABLE_VIEWER: [(&str, AccessRole); 1] = [("Table", AccessRole::Viewer)];

    fn fields<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        template::<R>(op, summary, description, true, FIELDS_TAG)
            .response_description::<404, ()>("Table not found")
    }

    fn select_fields<'a, R: OperationOutput>(
        op: TransformOperation<'a>,
        summary: &'a str,
        description: &'a str,
    ) -> TransformOperation<'a> {
        fields::<R>(op, summary, description)
            .response_description::<404, ()>("Table not found\n\nField not found")
    }

    pub fn create_field(op: TransformOperation) -> TransformOperation {
        fields::<Json<Field>>(op, "create_field", "Create a field in a table.")
            .response_description::<422, String>(INVALID_RANGE)
            .required_access(TABLE_OWNER)
    }

    pub fn update_field(op: TransformOperation) -> TransformOperation {
        select_fields::<Json<Field>>(op, "update_field", "Update a field's metadata in a table.")
            .response_description::<422, String>(INVALID_RANGE)
            .required_access(TABLE_OWNER)
    }

    pub fn delete_field(op: TransformOperation) -> TransformOperation {
        select_fields::<()>(
            op,
            "delete_field",
            "Delete a field and all cells in its respective column in the table.",
        )
        .required_access(TABLE_OWNER)
    }
    pub fn get_fields(op: TransformOperation) -> TransformOperation {
        select_fields::<Json<Vec<Field>>>(op, "get_fields", "Get all fields in a table.")
            .required_access(TABLE_VIEWER)
    }

    pub fn set_field_order(op: TransformOperation) -> TransformOperation {
        select_fields::<Json<Vec<Field>>>(
            op,
            "set_field_order",
            "Set the order of all fields in a table. Ordering numbers must go from `0` to `n-1` where `n` is the total number of fields",
        )
        .response_description::<400, String>(NO_DATA_IN_REQUEST_BODY)
        .response_description::<422, String>(&format!(
            "<field_id>: {FIELD_ID_NOT_FOUND}\n\n<field_id>: {INVALID_ORDERING}"
        ))
        .required_access(TABLE_OWNER)
    }
}

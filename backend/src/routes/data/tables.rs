use super::ApiState;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage, OnConstraint},
    model::data::{CreateTable, Table, UpdateTable},
    Id,
};
use axum::{
    extract::{Path, State},
    routing::{post, put},
    Json, Router,
};

const TABLE_NAME_CONFLICT: ErrorMessage =
    ErrorMessage::new_static("name", "Table name already used");


pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/tables", post(create_table).get(get_tables))
        .route("/tables/{table_id}", put(update_table).delete(delete_table))
}


/// Create an empty user table.
/// 
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::UnprocessableEntity`]:
///     - [`TABLE_NAME_CONFLICT`]
/// 
async fn create_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_table): Json<CreateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    let table = db::create_table(&pool, user_id, create_table)
        .await
        .on_constraint("meta_table_user_id_name_key", |_| {
            ApiError::unprocessable_entity([TABLE_NAME_CONFLICT])
        })?;

    Ok(Json(table))
}


/// Update a table's meta data.
/// 
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table not found
/// 
async fn update_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(update_table): Json<UpdateTable>,
) -> ApiResult<Json<Table>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let table = db::update_table(&pool, table_id, update_table).await?;

    Ok(Json(table))
}

/// Delete a table, including all fields and entries.
/// 
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// - [`ApiError::Forbidden`]: User does not have access to that table
/// - [`ApiError::NotFound`]: Table not found
/// 
async fn delete_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<()> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    db::delete_table(&pool, table_id).await?;

    Ok(())
}


/// Get all tables belonging to the user.
/// 
/// # Errors
/// - [`ApiError::Unauthorized`]: User not authenticated
/// 
async fn get_tables(State(ApiState { pool, .. }): State<ApiState>) -> ApiResult<Json<Vec<Table>>> {
    let user_id = db::debug_get_user_id(&pool).await?;

    let tables = db::get_tables(&pool, user_id).await?;
    Ok(Json(tables))
}
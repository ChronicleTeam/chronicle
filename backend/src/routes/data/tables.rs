use super::ApiState;
use crate::{
    db,
    error::{ApiError, ApiResult, ErrorMessage, OnConstraint},
    model::data::{CreateTable, Table, TableId, UpdateTable},
    Id,
};
use axum::{
    extract::{Path, State},
    routing::{post, put},
    Json, Router,
};
use axum_macros::debug_handler;

const TABLE_NAME_CONFLICT: ErrorMessage =
    ErrorMessage::new_static("name", "Table name already used");

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables",
        Router::new()
            .route("/", post(create_table).get(get_user_tables))
            .route("/{table_id}", put(update_table).delete(delete_table)),
    )
}

// #[debug_handler]
async fn create_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(create_table): Json<CreateTable>,
) -> ApiResult<Json<TableId>> {
    // TESTING
    let user_id = db::debug_get_user_id(&pool).await?;

    let table_id = db::create_table(&pool, user_id, create_table.name, create_table.description)
        .await
        .on_constraint("meta_table_user_id_name_key", |_| {
            ApiError::unprocessable_entity([TABLE_NAME_CONFLICT])
        })?;

    Ok(Json(TableId { table_id }))
}

// #[debug_handler]
async fn get_user_tables(
    State(ApiState { pool, .. }): State<ApiState>,
) -> ApiResult<Json<Vec<Table>>> {
    // TESTING

    let user_id = db::debug_get_user_id(&pool).await?;
    let tables = db::get_user_tables(&pool, user_id).await?;
    Ok(Json(tables))
}

async fn update_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(update_table): Json<UpdateTable>,
) -> ApiResult<Json<Table>> {
    let mut tx = pool.begin().await?;

    let user_id = db::debug_get_user_id(tx.as_mut()).await?;

    match db::check_table_ownership(tx.as_mut(), user_id, table_id).await? {
        db::Relation::Owned => {}
        db::Relation::NotOwned => return Err(ApiError::Forbidden),
        db::Relation::Absent => return Err(ApiError::NotFound),
    }

    let table = db::update_table(
        tx.as_mut(),
        table_id,
        update_table.name,
        update_table.description,
    )
    .await?;

    tx.commit().await?;

    Ok(Json(table))
}

async fn delete_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<()> {
    let mut tx = pool.begin().await?;
    // TESTING
    let user_id = db::debug_get_user_id(tx.as_mut()).await?;

    match db::check_table_ownership(tx.as_mut(), user_id, table_id).await? {
        db::Relation::Owned => {}
        db::Relation::NotOwned => return Err(ApiError::Forbidden),
        db::Relation::Absent => return Err(ApiError::NotFound),
    }

    db::delete_table(tx.as_mut(), table_id).await?;

    tx.commit().await?;
    
    Ok(())
}

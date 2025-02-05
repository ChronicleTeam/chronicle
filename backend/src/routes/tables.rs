use super::ApiState;
use crate::{
    error::{ApiError, ApiResult, OnConstraint},
    model::{CreateTable, Table, TableId, UpdateTable},
    db, Id,
};
use axum::{
    extract::{Path, State},
    routing::{post, put},
    Json, Router,
};
use axum_macros::debug_handler;

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
            ApiError::unprocessable_entity([("tables", "table name already used")])
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

    // TESTING
    let user_id = db::debug_get_user_id(tx.as_mut()).await?;

    let table_user_id = db::get_table_user_id_lock(tx.as_mut(), table_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if table_user_id != user_id {
        return Err(ApiError::Forbidden);
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

    let table_user_id = db::get_table_user_id_lock(tx.as_mut(), table_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if table_user_id != user_id {
        return Err(ApiError::Forbidden);
    }

    db::delete_table(tx.as_mut(), table_id).await?;

    tx.commit().await?;
    Ok(())
}


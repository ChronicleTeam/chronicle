use axum::{extract::State, routing::post, Json, Router};
use axum_macros::debug_handler;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::*, PgPool};

use crate::api::{
    error::{ApiError, ApiResult, OnContraint},
    Id,
};

use super::ApiState;

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables",
        Router::new().route("", post(create_table).get(get_all_tables)),
    )
}

// DTOs

#[derive(Serialize)]
struct Table {
    table_id: Id,
    name: String,
    description: String,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct TableId {
    table_id: Id,
}

#[derive(Deserialize)]
struct CreateTable {
    name: String,
    description: String,
}

#[derive(Deserialize, FromRow)]
struct InsertTable {
    table_id: Id,
    data_table_name: String,
}

// Route handlers

// #[debug_handler]
async fn create_table(
    State(ApiState { pool, .. }): State<ApiState>,
    Json(mut create_table): Json<CreateTable>,
) -> ApiResult<Json<TableId>> {
    // TESTING
    let user_id = debug_get_user_id(&pool).await?;

    let insert_table: InsertTable = sqlx::query_as(
        r#"
            INSERT INTO table_metadata ($1, $2, $3) 
            RETURNING table_id, data_table_name
        "#,
    )
    .bind(user_id)
    .bind(create_table.name)
    .bind(create_table.description)
    .fetch_one(&pool)
    .await
    .on_constraint("table_metadata_user_id_name_key", |_| {
        ApiError::unprocessable_entity([("table", "table name already used")])
    })?;

    // generate data table

    Ok(Json(TableId {
        table_id: insert_table.table_id,
    }))
}

#[debug_handler]
async fn get_all_tables(
    State(ApiState { pool, .. }): State<ApiState>,
) -> ApiResult<Json<Vec<Table>>> {
    todo!("Not implemented")
}

async fn debug_get_user_id(pool: &PgPool) -> Result<Id, sqlx::Error> {
    Ok(sqlx::query("SELECT user_id FROM app_user LIMIT 1;")
        .fetch_one(pool)
        .await?
        .get("user_id"))
}

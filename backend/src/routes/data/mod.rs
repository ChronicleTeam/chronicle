mod entries;
mod fields;
mod tables;

use crate::{db, error::ApiResult, model::data::TableData, routes::ApiState, Id};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};


/// [`Router`] for Data Management. Users must be authenticated to manage tables.
/// 
/// Routes paths:
/// - `/tables`: Create/read tables
/// - `/tables/{table_id}`: Delete/update a table
/// - `/tables/{table_id}/fields`: Create/read fields
/// - `/tables/{table_id}/fields/{field_id}`: Delete/update fields
/// - `/tables/{table_id}/entries`: Create/read entries
/// - `/tables/{table_id}/entries/{entry_id}`: Delete/update entries
/// - `/tables/{table_id}/data`: Get entire table data
pub fn router() -> Router<ApiState> {
    Router::new()
        .merge(tables::router())
        .merge(fields::router())
        .merge(entries::router())
        .route("/tables/{table_id}/data", get(get_table_data))
}


/// Get all the meta data, fields, and entries of a table.
/// 
/// Used for displaying the table in the user interface.
async fn get_table_data(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
) -> ApiResult<Json<TableData>> {
    let user_id = db::debug_get_user_id(&pool).await?;
    db::check_table_relation(&pool, user_id, table_id)
        .await?
        .to_api_result()?;

    let data_table = db::get_table_data(&pool, table_id).await?;

    Ok(Json(data_table))
}

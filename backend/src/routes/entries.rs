use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};

use crate::{model::CreateEntry, Id};

use super::ApiState;

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table_id}/entries",
        Router::new()
            .route("/", post(create_entry).get(get_entries))
            .route("/{entry_id}", patch(update_entry).delete(delete_entry)),
    )
}

async fn create_entry(
    State(ApiState { pool, .. }): State<ApiState>,
    Path(table_id): Path<Id>,
    Json(CreateEntry(create_entry)): Json<CreateEntry>,
) {
    todo!()
}

async fn get_entries() {
    todo!()
}

async fn update_entry() {
    todo!()
}

async fn delete_entry() {}

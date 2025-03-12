//! Route handlers for the Data Management feature.

mod entries;
mod fields;
mod tables;

use super::ApiState;
use axum::Router;

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
}

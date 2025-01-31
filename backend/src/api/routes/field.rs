use super::ApiState;
use crate::api::{error::ApiResult, Id};
use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{postgres::types::PgInterval, types::Decimal};

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table_id}/fields",
        Router::new().route("/", post(create_field)),
    )
}

#[derive(Deserialize)]
enum FieldOption {
    Text {
        is_required: bool,
    },
    Integer {
        is_required: bool,
        range_start: Option<i64>,
        ranger_end: Option<i64>,
    },
    Decimal {
        is_required: bool,
        range_start: Option<f64>,
        ranger_end: Option<f64>,
        scientific_notation: bool,
        number_precision: Option<i32>,
        number_scale: Option<i32>,
    },
    Money {
        is_required: bool,
        range_start: Option<Decimal>,
        range_end: Option<Decimal>,
    },
    Progress {
        total_steps: i32,
    },
    DateTime {
        is_required: bool,
        range_start: Option<DateTime<Utc>>,
        range_end: Option<DateTime<Utc>>,
        date_time_format: String,
    },
    Interval {
        is_required: bool,
        // range_start: Option<PgInterval>,
        // range_end: Option<PgInterval>,
    },
    WebLink {
        is_required: bool,
    },

    Email {
        is_required: bool,
    },
    Checkbox {
        default_value: bool,
    },
    Enumeration {
        is_required: bool,
        values: Vec<String>,
        default_value: i32,
    },
    CreationDate {
        is_required: bool,
        date_time_format: String,
    },
    ModificationDate {
        is_required: bool,
        date_time_format: String,
    },
    Image {
        is_required: bool,
    },
    File {
        is_required: bool,
    }
}

#[debug_handler]
async fn create_field(
    Path(table_id): Path<Id>,
    State(ApiState { pool, .. }): State<ApiState>,
    Json(field_option): Json<FieldOption>,
) -> ApiResult<()> {
    todo!("Not implemented")
}

use std::{collections::HashMap, fmt::format};

use super::ApiState;
use crate::{
    api::error::{ApiError, ApiResult},
    Id,
};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    routing::post,
    Json, Router,
};
use axum_macros::debug_handler;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgInterval, prelude::FromRow, types::Decimal};

pub(crate) fn router() -> Router<ApiState> {
    Router::new().nest(
        "/tables/{table_id}/fields",
        Router::new().route("/", post(create_field)),
    )
}

#[derive(Serialize)]
struct FieldId {
    field_id: Id,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum FieldOptions {
    Text {
        is_required: bool,
    },
    Integer {
        is_required: bool,
        range_start: Option<i64>,
        range_end: Option<i64>,
    },
    Decimal {
        is_required: bool,
        range_start: Option<f64>,
        range_end: Option<f64>,
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
    Checkbox,
    Enumeration {
        is_required: bool,
        values: HashMap<i32, String>,
        default_value: i32,
    },
    CreationDate {
        date_time_format: String,
    },
    ModificationDate {
        date_time_format: String,
    },
    Image {
        is_required: bool,
    },
    File {
        is_required: bool,
    },
}

impl FieldOptions {
    fn validate(&self) -> ApiResult<()> {
        match self {
            FieldOptions::Integer {
                range_start,
                range_end,
                ..
            } => validate_range(*range_start, *range_end),
            FieldOptions::Decimal {
                range_start,
                range_end,
                number_precision,
                number_scale,
                ..
            } => validate_range(*range_start, *range_end)
                .and_then(|_| {
                    if number_precision.map_or(true, |n| n >= 1) {
                        Err(anyhow!("number_precision must be >= 1").into())
                    } else {
                        Ok(())
                    }
                })
                .and_then(|_| {
                    if number_scale.map_or(true, |n| n >= 0) {
                        Err(anyhow!("number_scale must be >= 0").into())
                    } else {
                        Ok(())
                    }
                }),
            FieldOptions::Money {
                range_start,
                range_end,
                ..
            } => validate_range(*range_start, *range_end),
            FieldOptions::DateTime {
                range_start,
                range_end,
                date_time_format,
                ..
            } => validate_range(*range_start, *range_end),
            FieldOptions::Interval { .. } => Ok(()),
            FieldOptions::Enumeration {
                values,
                default_value,
                ..
            } => {
                if !values.contains_key(default_value) {
                    Err(anyhow!("enumeration field default value does not map to a value").into())
                } else {
                    Ok(())
                }
            }
            FieldOptions::CreationDate { date_time_format } => Ok(()),
            FieldOptions::ModificationDate { date_time_format } => Ok(()),
            _ => Ok(()),
        }
    }
}

#[derive(FromRow)]
struct InsertField {
    field_id: Id,
    data_field_name: String,
}

#[derive(FromRow)]
struct SelectDataTableName {
    data_table_name: String,
}

#[debug_handler]
async fn create_field(
    Path(table_id): Path<Id>,
    State(ApiState { pool, .. }): State<ApiState>,
    Json(field_options): Json<FieldOptions>,
) -> ApiResult<Json<FieldId>> {
    field_options.validate()?;

    let SelectDataTableName { data_table_name } = sqlx::query_as(
        r#"
            SELECT data_table_name
            FROM table_metadata
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(&pool)
    .await?;

    let mut transaction = pool.begin().await?;

    let InsertField {
        field_id,
        data_field_name,
    } = sqlx::query_as(
        r#"
            INSERT INTO table_field (table_id, field_options)
            VALUES ($1, $2)
            RETURNING field_id, data_field_name
        "#,
    )
    .bind(table_id)
    .bind(sqlx::types::Json(field_options.clone()))
    .fetch_one(transaction.as_mut())
    .await?;

    let column_type = match field_options {
        FieldOptions::Text { .. } => "TEXT",
        FieldOptions::Integer { .. } => "BIGINT",
        FieldOptions::Decimal { .. } => "DOUBLE",
        FieldOptions::Money { .. } => "numeric_money",
        FieldOptions::Progress { .. } => "INT NOT NULL DEFAULT 0",
        FieldOptions::DateTime { .. } => "TIMESTAMPTZ",
        FieldOptions::Interval { .. } => "INTERVAL",
        FieldOptions::WebLink { .. } => "COLLATE case_insensitive TEXT",
        FieldOptions::Email { .. } => "COLLATE case_insensitive TEXT",
        FieldOptions::Checkbox => "BOOLEAN NOT NULL DEFAULT FALSE",
        FieldOptions::Enumeration { .. } => "INT",
        FieldOptions::CreationDate { .. } => {
            transaction.commit().await?;
            return Ok(Json(FieldId { field_id }));
        }
        FieldOptions::ModificationDate { .. } => {
            transaction.commit().await?;
            return Ok(Json(FieldId { field_id }));
        }
        FieldOptions::Image { .. } => Err(anyhow!("Not implemented"))?,
        FieldOptions::File { .. } => Err(anyhow!("Not implemented"))?,
    };

    // data_table_name and data_field_name generated by database NO INJECTION POSSIBLE
    sqlx::query(&format!(
        r#"
            ALTER TABLE {data_table_name}
            ADD COLUMN {data_field_name} {column_type}
        "#,
    ))
    .execute(transaction.as_mut()).await?;

    transaction.commit().await?;
    return Ok(Json(FieldId { field_id }));
}

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
        Err(ApiError::unprocessable_entity([(
            "range",
            "range start bound is greater than end bound",
        )]))
    }
}

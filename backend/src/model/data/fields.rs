use crate::Id;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use sqlx::{FromRow, types::Json};
use std::{collections::HashMap, fmt};

/// Table field entity.
#[derive(Debug, Clone, Serialize, PartialEq, FromRow, JsonSchema)]
pub struct Field {
    pub field_id: Id,
    pub table_id: Id,
    pub name: String,
    pub ordering: i32,
    #[schemars(with = "FieldKind")]
    pub field_kind: Json<FieldKind>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// The field kind and associated options.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type")]
pub enum FieldKind {
    /// Raw text data.
    Text { is_required: bool },
    /// An integer.
    Integer {
        is_required: bool,
        range_start: Option<i64>,
        range_end: Option<i64>,
    },
    /// A floating-point number.
    Float {
        is_required: bool,
        range_start: Option<f64>,
        range_end: Option<f64>,
    },
    /// A type for fixed precision money.
    Money {
        is_required: bool,
        range_start: Option<Decimal>,
        range_end: Option<Decimal>,
    },
    /// A discrete progress ratio.
    Progress { total_steps: i64 },
    /// An ISO 8601 date and time.
    DateTime {
        is_required: bool,
        range_start: Option<DateTime<Utc>>,
        range_end: Option<DateTime<Utc>>,
    },
    /// A URL.
    WebLink { is_required: bool },
    /// A true or false value.
    Checkbox,
    /// A value out of a list of possible text values.
    Enumeration {
        is_required: bool,
        #[schemars(with = "HashMap<i64, String>")]
        #[serde_as(as = "HashMap<DisplayFromStr, _>")] // https://github.com/serde-rs/json/issues/496
        values: HashMap<i64, String>,
        default_value: i64,
    },
}

impl FieldKind {
    /// Map the field kind to the PostgreSQL data type.
    pub fn get_sql_type(&self) -> &'static str {
        match self {
            FieldKind::Text { .. } => "TEXT",
            FieldKind::Integer { .. } => "BIGINT",
            FieldKind::Float { .. } => "DOUBLE PRECISION",
            FieldKind::Money { .. } => "numeric_money",
            FieldKind::Progress { .. } => "BIGINT NOT NULL DEFAULT 0",
            FieldKind::DateTime { .. } => "TIMESTAMPTZ",
            FieldKind::WebLink { .. } => "TEXT COLLATE case_insensitive",
            FieldKind::Checkbox => "BOOLEAN NOT NULL DEFAULT FALSE",
            FieldKind::Enumeration { .. } => "BIGINT",
        }
    }
}

/// Create field request.
#[derive(Debug, Clone, Deserialize, PartialEq, FromRow, JsonSchema)]
pub struct CreateField {
    pub name: String,
    pub field_kind: FieldKind,
}

/// Update field request.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateField {
    pub name: String,
    pub field_kind: FieldKind,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SelectField {
    pub table_id: Id,
    pub field_id: Id,
}

/// Set the field order request.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetFieldOrder(pub HashMap<Id, i32>);

/// DTO for when a field's ID and field kind is needed.
#[derive(Debug, FromRow)]
pub struct FieldMetadata {
    pub field_id: Id,
    pub field_kind: Json<FieldKind>,
}

impl FieldMetadata {
    pub fn from_field(field: Field) -> Self {
        Self {
            field_id: field.field_id,
            field_kind: field.field_kind,
        }
    }
}

/// Database identifier of the actual SQL table column that a user field points to.
#[derive(Debug)]
pub struct FieldIdentifier {
    field_id: Id,
}
impl FieldIdentifier {
    pub fn new(field_id: Id) -> Self {
        Self { field_id }
    }
    pub fn unquote(&self) -> String {
        format!("f{}", self.field_id)
    }
}
impl fmt::Display for FieldIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""f{}""#, self.field_id)
    }
}

use std::fmt;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};

use crate::{Id, model::data::FieldKind};

/// Chart axis entity.
#[derive(Debug, Serialize, Deserialize, FromRow, JsonSchema, PartialEq)]
pub struct Axis {
    pub axis_id: Id,
    pub chart_id: Id,
    pub field_id: Id,
    pub axis_kind: AxisKind,
    pub aggregate: Option<Aggregate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// The kind of axis for constructing the actual chart.
#[derive(
    Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type, JsonSchema,
)]
#[sqlx(type_name = "axis_kind")]
pub enum AxisKind {
    X,
    Y,
    Color,
    Size,
    Tooltip,
    Label,
    Detail,
}

/// The aggregate function of the axis.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, JsonSchema, PartialEq, Eq, Hash,
)]
#[sqlx(type_name = "aggregate")]
pub enum Aggregate {
    Sum,
    Average,
    Min,
    Max,
    Count,
}

impl Aggregate {
    /// Get the SQL function of this aggregate.
    pub fn get_sql_aggregate(&self) -> &'static str {
        match self {
            Aggregate::Sum => "SUM",
            Aggregate::Average => "AVG",
            Aggregate::Min => "MIN",
            Aggregate::Max => "MAX",
            Aggregate::Count => "COUNT",
        }
    }

    /// Get the SQL type of this aggregate based on field kind.
    pub fn get_sql_type(&self, field_kind: &FieldKind) -> &'static str {
        match self {
            Aggregate::Sum | Aggregate::Average => match field_kind {
                FieldKind::Float { .. } => "DOUBLE PRECISION",
                _ => "NUMERIC",
            },
            Aggregate::Min | Aggregate::Max => field_kind.get_sql_type(),
            Aggregate::Count => "BIGINT",
        }
    }
}

/// Create axis request.
#[derive(Debug, Clone, Deserialize, FromRow, JsonSchema, PartialEq, Eq, Hash)]
pub struct CreateAxis {
    pub field_id: Id,
    pub axis_kind: AxisKind,
    pub aggregate: Option<Aggregate>,
}

/// Set a chart's axis request.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetAxes(pub Vec<CreateAxis>);

/// An axis and its associated field.
#[derive(Debug, Serialize, Deserialize, FromRow, JsonSchema, PartialEq)]
pub struct AxisField {
    #[sqlx(flatten)]
    pub axis: Axis,
    pub field_name: String,
    #[schemars(with = "FieldKind")]
    pub field_kind: Json<FieldKind>,
}

/// Database identifier of the actual SQL view column that a user axis points to.
#[derive(Debug)]
pub struct AxisIdentifier {
    axis_id: Id,
}
impl AxisIdentifier {
    pub fn new(axis_id: Id) -> Self {
        Self { axis_id }
    }
    pub fn unquoted(&self) -> String {
        format!("a{}", self.axis_id)
    }
}
impl fmt::Display for AxisIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""a{}""#, self.axis_id)
    }
}

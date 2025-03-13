use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};

use crate::{model::data::FieldKind, Id};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Axis {
    pub axis_id: Id,
    pub chart_id: Id,
    pub field_id: Id,
    pub axis_kind: AxisKind,
    pub aggregate: Option<Aggregate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type)]
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

#[derive(Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "aggregate")]
pub enum Aggregate {
    Sum,
    Average,
    Min,
    Max,
    Count,
}

impl Aggregate {
    pub fn get_sql_aggregate(&self) -> &'static str {
        match self {
            Aggregate::Sum => "SUM",
            Aggregate::Average => "AVG",
            Aggregate::Min => "MIN",
            Aggregate::Max => "MAX",
            Aggregate::Count => "COUNT",
        }
    }

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

#[derive(Deserialize)]
pub struct CreateAxis {
    pub field_id: Id,
    pub axis_kind: AxisKind,
    pub aggregate: Option<Aggregate>,
}

#[derive(Deserialize)]
pub struct SetAxes(pub Vec<CreateAxis>);

#[derive(Serialize, FromRow)]
pub struct AxisField {
    #[sqlx(flatten)]
    pub axis: Axis,
    pub field_name: String,
    pub field_kind: Json<FieldKind>,
}


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

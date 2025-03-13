use crate::{model::Cell, Id};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::{collections::HashMap, fmt};

use super::AxisField;

#[derive(Serialize, FromRow)]
pub struct Chart {
    pub chart_id: Id,
    pub dashboard_id: Id,
    pub table_id: Id,
    pub title: String,
    pub chart_kind: ChartKind,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "chart_kind")]
pub enum ChartKind {
    Table,
    Bar,
    Line,
}

#[derive(Deserialize)]
pub struct CreateChart {
    pub table_id: Id,
    pub title: String,
    pub chart_kind: ChartKind,
}


#[derive(Deserialize)]
pub struct UpdateChart {
    pub title: String,
    pub chart_kind: ChartKind,
}

#[derive(Serialize)]
pub struct ChartData {
    pub chart: Chart,
    pub axes: Vec<AxisField>,
    pub cells: Vec<HashMap<Id, Option<Cell>>>,
}


pub struct ChartIdentifier {
    chart_id: Id,
    schema: String,
}
impl ChartIdentifier {
    pub fn new(chart_id: Id, schema: &str) -> Self {
        Self {
            chart_id,
            schema: schema.to_string(),
        }
    }
}
impl fmt::Display for ChartIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""{}"."c{}""#, self.schema, self.chart_id)
    }
}

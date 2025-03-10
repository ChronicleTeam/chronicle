mod axes;

pub use axes::*;

use crate::{model::CellMap, Id};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::collections::HashMap;

#[derive(Serialize, FromRow)]
pub struct Chart {
    pub chart_id: Id,
    pub dashboard_id: Id,
    pub title: String,
    pub chart_kind: ChartKind,
    #[serde(skip)]
    pub data_view_name: String,
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
    pub title: String,
    pub chart_kind: ChartKind,
}

#[derive(Serialize)]
pub struct ChartData {
    pub chart: Chart,
    pub axis_data_map: HashMap<Id, AxisData>,
    pub cells: Vec<CellMap>,
}

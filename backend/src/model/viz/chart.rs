use crate::{
    model::data::{CellEntry, Field},
    Id,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::collections::HashMap;

#[derive(Serialize, FromRow)]
pub struct Chart {
    pub chart_id: Id,
    pub dashboard_id: Id,
    pub table_id: Id,
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
    Bar,
    Line,
}

#[derive(Deserialize)]
pub struct CreateChart {
    pub table_id: Id,
    pub title: String,
    pub chart_kind: ChartKind,
    pub axes: Vec<CreateAxis>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Axis {
    pub axis_id: Id,
    pub chart_id: Id,
    pub field_id: Id,
    pub axis_kind: AxisKind,
    pub aggregate: Option<Aggregate>,
    #[serde(skip)]
    pub data_item_name: String,
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
    // Detail
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "aggregate")]
pub enum Aggregate {
    Sum,
    Average,
    Count,
}

#[derive(Deserialize)]
pub struct CreateAxis {
    pub field_id: Id,
    pub axis_kind: AxisKind,
    pub aggregate: Option<Aggregate>,
}

#[derive(Serialize)]
pub struct ChartData {
    pub chart: Chart,
    pub axis_data_map: HashMap<Id, AxisData>,
    pub cells: Vec<CellEntry>,
}

#[derive(Serialize)]
pub struct AxisData {
    pub axis: Axis,
    pub field: Field,
}

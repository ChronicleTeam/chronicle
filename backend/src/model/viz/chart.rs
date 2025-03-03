use crate::{
    model::data::{Cell, Field},
    Id,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ChartData {
    pub chart_id: Id,
    pub dashboard: Id,
    pub title: String,
    pub x_axis: AxisData,
    pub y_axis: AxisData,
    pub marks: HashMap<MarkKind, AxisData>,
    pub plot_kind: ChartKind,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
pub struct Chart {
    pub chart_id: Id,
    pub dashboard: Id,
    pub title: String,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub chart_kind: ChartKind,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(FromRow)]
pub struct Mark {
    pub chart_id: Id,
    pub mark_kind: MarkKind,
    pub axis: Axis,
}

#[derive(Deserialize)]
pub struct CreateChart {
    pub table_id: Id,
    pub title: String,
    pub chart_kind: ChartKind,
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub marks: HashMap<MarkKind, Axis>,
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "chart_kind")]
pub enum ChartKind {
    Bar,
    Line,
}

#[derive(Serialize)]
pub struct AxisData {
    pub data: Vec<Cell>,
    pub field: Field,
    pub aggregate: Option<Aggregate>,
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "axis")]
pub struct Axis {
    pub field_id: Id,
    pub aggregate: Option<Aggregate>,
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "aggregate")]
pub enum Aggregate {
    Sum,
    Average,
    Count,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(type_name = "mark_kind")]
pub enum MarkKind {
    Color,
    Size,
    Tooltip,
    Label,
    // Detail
}

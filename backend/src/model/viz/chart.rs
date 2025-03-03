use crate::{
    model::data::{Cell, Field},
    Id,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ChartData {
    dashboard: Id,
    table_id: Id,
    title: String,
    x_axis: AxisData,
    y_axis: AxisData,
    marks: HashMap<MarkKind, AxisData>,
    plot_kind: ChartKind,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
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
    data: Vec<Cell>,
    field: Field,
    aggregate: Option<Aggregate>,
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "axis")]
pub struct Axis {
    field_id: Id,
    aggregate: Option<Aggregate>,
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

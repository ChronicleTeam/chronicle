use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::Id;
use super::CalculatedField;

// #[derive(Serialize)]
// pub struct Chart {
//     table_id: Id,
//     x_axis: CalculatedField,
//     y_axis: CalculatedField,
//     marks: HashSet<Mark>,
//     plot_kind: ChartKind,
//     data: (),
//     created_at: DateTime<Utc>,
//     updated_at: Option<DateTime<Utc>>
// }


#[derive(Serialize, Deserialize)]
pub enum ChartKind {
    Bar,
    Line,
}

#[derive(Deserialize)]
pub enum Mark {
    Color,
    Size(f32),
    Tooltip,
    Label,
    // Detail
}


#[derive(Deserialize)]
pub struct CreateChart {
    table_id: Id,
    title: String,
    x_axis: CalculatedField,
    y_axis: CalculatedField,
    marks: Vec<(Mark, CalculatedField)>,
    chart_kind: ChartKind,
}
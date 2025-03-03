use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::Id;
use super::CalculatedField;

#[derive(Serialize)]
pub struct Plot {
    table_id: Id,
    columns: Vec<CalculatedField>,
    rows: Vec<CalculatedField>,
    plot_kind: PlotKind,
    data: (),
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>
}


#[derive(Serialize, Deserialize)]
pub enum PlotKind {
    Bar,
    Line,
}


#[derive(Deserialize)]
pub struct CreatePlot {
    table_id: Id,
    columns: Vec<CalculatedField>,
    rows: Vec<CalculatedField>,
    plot_kind: PlotKind,

}
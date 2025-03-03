mod chart;

pub use {chart::*};
use {crate::Id, serde::{Deserialize, Serialize}};


#[derive(Serialize, Deserialize)]
pub enum Aggregate {
    Sum,
    Average,
    Count,
}

#[derive(Serialize, Deserialize)]
pub struct CalculatedField {
    field_id: Id,
    aggregate: Option<Aggregate>,
}
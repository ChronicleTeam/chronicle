mod entries;
mod fields;
mod tables;

use serde::Serialize;
pub use {entries::*, fields::*, tables::*};

#[derive(Serialize)]
pub struct TableData {
    pub table: Table,
    pub fields: Vec<Field>,
    pub entries: Vec<Entry>,
}

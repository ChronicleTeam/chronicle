mod entries;
mod fields;
mod tables;

use {crate::Id, serde::Serialize};
pub use {entries::*, fields::*, tables::*};

#[derive(Serialize)]
pub struct DataTable {
    pub fields: Vec<Field>,
    pub entries: Vec<Entry>,
}

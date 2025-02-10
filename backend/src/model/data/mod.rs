mod entries;
mod fields;
mod tables;

use {crate::Id, serde::Serialize};
pub use {entries::*, fields::*, tables::*};

#[derive(Serialize)]
pub struct DataTable {
    fields: Vec<(Id, FieldOptions)>,
    entries: Vec<(Id, Vec<Cell>)>,
}

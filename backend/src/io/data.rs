use std::path::Path;

use umya_spreadsheet::{reader::xlsx, XlsxError};



pub fn import_from_xlsx(path: &Path) -> Result<(), XlsxError> {

    let mut book = xlsx::lazy_read(path)?;
    for sheet in book.get_sheet_collection() {
        sheet.get_highest_column_and_row();
    }

    todo!()
}
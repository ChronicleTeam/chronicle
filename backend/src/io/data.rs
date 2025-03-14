use std::path::Path;

use umya_spreadsheet::{reader::xlsx, XlsxError};



pub fn import_from_xlsx(path: &Path) -> Result<(), XlsxError> {

    let mut book = xlsx::lazy_read(path)?;

    

    todo!()
}
use crate::model::{
    data::{CreateField, CreateTable, CreateTableData, FieldKind},
    Cell,
};
use std::collections::HashSet;
use umya_spreadsheet::Spreadsheet;

const EXCEL_IMPORT_TABLE_DESCRIPTION: &str = "This table was imported from Excel";

pub fn import_table_from_excel(spreadsheet: Spreadsheet) -> Vec<CreateTableData> {
    let mut tables: Vec<CreateTableData> = Vec::new();

    for sheet in spreadsheet.get_sheet_collection() {
        let table = CreateTable {
            name: sheet.get_name().to_string(),
            description: EXCEL_IMPORT_TABLE_DESCRIPTION.to_string(),
        };
        let mut fields: Vec<CreateField> = Vec::new();
        let mut entries: Vec<Vec<Cell>> = Vec::new();

        let (columns, rows) = sheet.get_highest_column_and_row();

        let mut fields_names = HashSet::new();
        for col in 1..=columns {
            let original_name = sheet.get_value((col, 1));
            let mut name = original_name.clone();
            let mut count = 1;
            while fields_names.contains(&name) {
                name = format!("{original_name}{count}");
                count += 1;
            }
            fields_names.insert(name.clone());

            fields.push(CreateField {
                name,
                field_kind: FieldKind::Text { is_required: false },
            });
        }

        for row in 2..=rows {
            let mut entry: Vec<Cell> = Vec::new();
            for col in 1..=columns {
                let value = sheet.get_value((col, row));
                let value = if value.is_empty() { Cell::Null } else { Cell::String(value) };
                entry.push(value);
            }
            entries.push(entry);
        }

        tables.push(CreateTableData {
            table,
            fields,
            entries,
        });
    }

    tables
}

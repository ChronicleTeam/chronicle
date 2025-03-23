use crate::{
    model::{
        data::{CreateField, CreateTable, CreateTableData, Field, FieldKind, TableData},
        Cell,
    },
    Id,
};
use itertools::Itertools;
use num_traits::ToBytes;
use std::{
    collections::{HashMap, HashSet},
    io,
};
use umya_spreadsheet::Spreadsheet;

const EXCEL_IMPORT_TABLE_DESCRIPTION: &str = "This table was imported from Excel";
const CSV_IMPORT_TABLE_DESCRIPTION: &str = "This table was imported from CSV";

pub fn import_table_from_excel(spreadsheet: Spreadsheet) -> Vec<CreateTableData> {
    let mut tables: Vec<CreateTableData> = Vec::new();

    for sheet in spreadsheet.get_sheet_collection() {
        let table = CreateTable {
            name: sheet.get_name().to_string(),
            description: EXCEL_IMPORT_TABLE_DESCRIPTION.to_string(),
        };
        let mut fields: Vec<CreateField> = Vec::new();
        let (columns, rows) = sheet.get_highest_column_and_row();

        let mut fields_names = HashSet::new();
        for col in 1..=columns {
            let original_name = sheet.get_value((col, 1));
            let mut name = original_name.clone();
            let mut count = 1;
            while fields_names.contains(&name) {
                name = format!("{original_name} ({count})");
                count += 1;
            }
            fields_names.insert(name.clone());

            fields.push(CreateField {
                name,
                field_kind: FieldKind::Text { is_required: false },
            });
        }

        let entries: Vec<Vec<Cell>> = (2..=rows)
            .map(|row| {
                (1..columns)
                    .map(|col| {
                        let value = sheet.get_value((col, row));
                        if value.is_empty() {
                            Cell::Null
                        } else {
                            Cell::String(value)
                        }
                    })
                    .collect()
            })
            .collect();

        tables.push(CreateTableData {
            table,
            fields,
            entries,
        });
    }

    tables
}

pub fn export_table_to_excel(
    spreadsheet: &mut Spreadsheet,
    TableData {
        table,
        fields,
        entries,
    }: TableData,
) {
    let mut sheet_name = table.name.clone();

    let mut i = 1;
    let sheet = loop {
        match spreadsheet.new_sheet(&sheet_name) {
            Ok(sheet) => break sheet, // Exit the loop and return the sheet
            Err(_) => {
                sheet_name = format!("{} ({i})", table.name);
                i += 1;
            }
        }
    };

    let fields: HashMap<Id, Field> = fields
        .into_iter()
        .map(|field| (field.field_id, field))
        .collect();

    for field in fields.values() {
        let col = field.ordering as u32 + 1;
        sheet
            .get_cell_mut((col, 0))
            .set_value_string(field.name.clone());
    }

    for (row, entry) in entries.into_iter().enumerate() {
        let row = row as u32 + 2;
        for (field_id, cell) in entry.cells.into_iter() {
            if let Cell::Null = cell {
                continue;
            }

            let field = fields.get(&field_id).unwrap();
            let col = field.ordering as u32 + 1;
            let sheet_cell = sheet.get_cell_mut((col, row));

            match (&field.field_kind.0, cell) {
                (FieldKind::Text { .. } | FieldKind::WebLink { .. }, Cell::String(v)) => {
                    sheet_cell.set_value_string(v)
                }
                (FieldKind::Integer { .. } | FieldKind::Progress { .. }, Cell::Integer(v)) => {
                    sheet_cell.set_value_number(v as f64)
                }
                (FieldKind::Float { .. }, Cell::Float(v)) => sheet_cell.set_value_number(v),
                (FieldKind::Money { .. }, Cell::Decimal(v)) => {
                    sheet_cell.set_value_string(v.to_string())
                }
                (FieldKind::DateTime { .. }, Cell::DateTime(v)) => {
                    sheet_cell.set_value_string(v.to_rfc3339())
                }
                (FieldKind::Checkbox, Cell::Boolean(v)) => sheet_cell.set_value_bool(v),
                (FieldKind::Enumeration { values, .. }, Cell::Integer(v)) => {
                    sheet_cell.set_value_string(values.get(&v).unwrap())
                }
                _ => unreachable!(),
            };
        }
    }
}

pub fn import_table_from_csv<R>(
    mut csv_reader: csv::Reader<R>,
    name: &str,
) -> csv::Result<CreateTableData>
where
    R: std::io::Read,
{
    let table = CreateTable {
        name: name.to_string(),
        description: CSV_IMPORT_TABLE_DESCRIPTION.to_string(),
    };

    let mut fields: Vec<CreateField> = Vec::new();

    let mut fields_names = HashSet::new();
    for original_name in csv_reader.headers()? {
        let mut name = original_name.to_string();
        let mut count = 1;
        while fields_names.contains(&name) {
            name = format!("{original_name} ({count})");
            count += 1;
        }
        fields_names.insert(name.clone());

        fields.push(CreateField {
            name,
            field_kind: FieldKind::Text { is_required: false },
        });
    }

    let entries: Vec<Vec<Cell>> = csv_reader
        .records()
        .map(|record| {
            Ok(record?
                .into_iter()
                .map(|value| {
                    if value.is_empty() {
                        Cell::Null
                    } else {
                        Cell::String(value.to_string())
                    }
                })
                .collect())
        })
        .collect::<csv::Result<_>>()?;

    Ok(CreateTableData {
        table,
        fields,
        entries,
    })
}

pub fn export_table_to_csv<W>(
    mut csv_writer: csv::Writer<W>,
    TableData {
        table,
        fields,
        entries,
    }: TableData,
) -> csv::Result<()>
where
    W: io::Write,
{
    csv_writer.write_record(
        fields
            .iter()
            .sorted_by_key(|field| field.ordering)
            .map(|field| field.name.clone()),
    )?;

    let fields: HashMap<_, _> = fields
        .into_iter()
        .map(|field| (field.field_id, field))
        .collect();

    for entry in entries {
        csv_writer.write_record(
            entry
                .cells
                .into_iter()
                .sorted_by_key(|(field_id, _)| fields.get(&field_id).unwrap().ordering)
                .map(|(_, cell)| match cell {
                    Cell::Integer(v) => v.to_string(),
                    Cell::Float(v) => v.to_string(),
                    Cell::Decimal(v) => v.to_string(),
                    Cell::Boolean(v) => v.to_string(),
                    Cell::DateTime(v) => v.to_rfc3339(),
                    Cell::String(v) => v,
                    Cell::Null => String::new(),
                }),
        )?;
    }

    Ok(())
}

use crate::{
    Id,
    model::{
        Cell,
        data::{CreateField, CreateTable, CreateTableData, Field, FieldKind, TableData},
    },
};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    io,
};
use umya_spreadsheet::Spreadsheet;

const EXCEL_IMPORT_TABLE_DESCRIPTION: &str = "This table was imported from Excel";
const CSV_IMPORT_TABLE_DESCRIPTION: &str = "This table was imported from CSV";

/// Create the [CreateTableData] DTOs from an Excel spreadsheet.
pub fn import_table_from_excel(spreadsheet: Spreadsheet) -> Vec<CreateTableData> {
    let mut tables: Vec<CreateTableData> = Vec::new();

    for sheet in spreadsheet.get_sheet_collection() {
        let table = CreateTable {
            parent_id: None,
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
                (1..=columns)
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

/// Convert a [TableData] DTO into the Excel spreadsheet. Currently, child tables are ignored.
pub fn export_table_to_excel(
    spreadsheet: &mut Spreadsheet,
    TableData {
        table,
        fields,
        entries,
        ..
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

            match cell {
                Cell::String(v) => sheet_cell.set_value_string(v),
                Cell::Integer(v) => {
                    if let FieldKind::Enumeration { values, .. } = &field.field_kind.0 {
                        sheet_cell.set_value_string(values.get(&v).unwrap())
                    } else {
                        sheet_cell.set_value_number(v as f64)
                    }
                }
                Cell::Float(v) => sheet_cell.set_value_number(v),
                Cell::Decimal(v) => sheet_cell.set_value_string(v.to_string()),
                Cell::DateTime(v) => sheet_cell.set_value_string(v.to_rfc3339()),
                Cell::Boolean(v) => sheet_cell.set_value_bool(v),
                Cell::Null => unreachable!(),
            };
        }
    }
}

/// Create a [CreateTableData] DTO from a CSV file.
pub fn import_table_from_csv<R>(
    mut csv_reader: csv::Reader<R>,
    name: &str,
) -> csv::Result<CreateTableData>
where
    R: std::io::Read,
{
    let table = CreateTable {
        parent_id: None,
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

/// Convert a [TableData] DTO into the CSV file. Currently, child tables are ignored.
pub fn export_table_to_csv<W>(
    mut csv_writer: csv::Writer<W>,
    TableData {
        table: _,
        fields,
        entries,
        ..
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
                .sorted_by_key(|(entry_id, _)| fields.get(entry_id).unwrap().ordering)
                .map(|(entry_id, cell)| match cell {
                    Cell::Integer(v) => {
                        if let FieldKind::Enumeration { values, .. } =
                            &fields.get(&entry_id).unwrap().field_kind.0
                        {
                            values.get(&v).unwrap().clone()
                        } else {
                            v.to_string()
                        }
                    }
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::{
        error::IntoAnyhow,
        model::{
            Cell,
            data::{
                CreateField, CreateTable, CreateTableData, Entry, Field, FieldKind, Table,
                TableData,
            },
        },
    };
    use anyhow::{Ok, Result};
    use chrono::Utc;
    use sqlx::types::Json;
    use std::collections::HashMap;
    use umya_spreadsheet::{new_file_empty_worksheet, reader, writer};

    #[test]
    fn import_table_from_excel() -> Result<()> {
        let path = std::path::Path::new("./testing/import.xlsx");
        let spreadsheet = reader::xlsx::read(path)?;

        let test_data = crate::io::import_table_from_excel(spreadsheet);

        let base_data = [CreateTableData {
            table: CreateTable {
                parent_id: None,
                name: "Sheet1".into(),
                description: "This table was imported from Excel".into(),
            },
            fields: vec![
                CreateField {
                    name: "Field 1".into(),
                    field_kind: crate::model::data::FieldKind::Text { is_required: false },
                },
                CreateField {
                    name: "Field 2".into(),
                    field_kind: crate::model::data::FieldKind::Text { is_required: false },
                },
            ],
            entries: vec![
                vec![
                    Cell::String("c1".to_string()),
                    Cell::String("1".to_string()),
                ],
                vec![
                    Cell::String("c2".to_string()),
                    Cell::String("2.343".to_string()),
                ],
                vec![
                    Cell::String("c3".to_string()),
                    Cell::String("crab".to_string()),
                ],
                vec![
                    Cell::String("c4".to_string()),
                    Cell::String("egg".to_string()),
                ],
            ],
        }];

        assert_eq!(
            base_data.len(),
            test_data.len(),
            "Length of imported values is not the same length."
        );

        let cmp_tables = base_data.iter().zip(&test_data);
        for cmp in cmp_tables {
            assert_eq!(cmp.0.table, cmp.1.table, "Tables do not match.");

            let cmp_fields = cmp.0.fields.iter().zip(&cmp.1.fields);

            for field in cmp_fields {
                assert_eq!(field.0, field.1, "Fields do not match.");
            }

            let cmp_entries = cmp.0.entries.iter().zip(&cmp.1.entries);

            for entry in cmp_entries {
                assert_eq!(entry.0, entry.1);
            }
        }

        Ok(())
    }

    #[test]
    fn import_table_from_csv() -> Result<()> {
        let path = std::path::Path::new("./testing/import.csv");
        let csv = csv::Reader::from_path(path)?;

        let test_data = crate::io::import_table_from_csv(csv, "Sheet1")?;

        let base_data = CreateTableData {
            table: CreateTable {
                parent_id: None,
                name: "Sheet1".into(),
                description: "This table was imported from CSV".into(),
            },
            fields: vec![
                CreateField {
                    name: "Field".into(),
                    field_kind: crate::model::data::FieldKind::Text { is_required: false },
                },
                CreateField {
                    name: "Value".into(),
                    field_kind: crate::model::data::FieldKind::Text { is_required: false },
                },
            ],
            entries: vec![
                vec![
                    Cell::String("c1".to_string()),
                    Cell::String("2".to_string()),
                ],
                vec![
                    Cell::String("c2".to_string()),
                    Cell::String("4.239".to_string()),
                ],
                vec![
                    Cell::String("c3".to_string()),
                    Cell::String("crab".to_string()),
                ],
                vec![
                    Cell::String("c4".to_string()),
                    Cell::String("soda".to_string()),
                ],
            ],
        };

        assert_eq!(base_data.table, test_data.table, "Tables do not match.");

        let cmp_fields = base_data.fields.iter().zip(&test_data.fields);

        for field in cmp_fields {
            assert_eq!(field.0, field.1, "Fields do not match.");
        }

        let cmp_entries = base_data.entries.iter().zip(&test_data.entries);

        for entry in cmp_entries {
            assert_eq!(entry.0, entry.1);
        }

        Ok(())
    }

    #[test]
    fn export_table_to_excel() -> Result<()> {
        // Base data gen
        let now = Utc::now();
        let table_id = 123;

        let mut entries1 = HashMap::new();
        entries1.insert(421, Cell::String("help1".to_string()));

        let mut entries2 = HashMap::<i32, Cell>::new();
        entries2.insert(213, Cell::String("help2".to_string()));

        let entries = vec![
            Entry {
                entry_id: 21,
                parent_id: None,
                created_at: now,
                updated_at: None,
                cells: entries1,
            },
            Entry {
                entry_id: 210,
                parent_id: None,
                created_at: now,
                updated_at: None,
                cells: entries2,
            },
        ];
        let table_data = TableData {
            table: Table {
                table_id,
                name: "Dummy Table".to_string(),
                description: "Exported to xlsx".to_string(),
                parent_id: None,
                created_at: now,
                updated_at: None,
            },
            fields: vec![
                Field {
                    field_id: 421,
                    name: "Field 1".into(),
                    table_id,
                    ordering: 0,
                    field_kind: Json(FieldKind::Text { is_required: false }),
                    created_at: now,
                    updated_at: None,
                },
                Field {
                    field_id: 213,
                    name: "Field 2".into(),
                    table_id,
                    ordering: 0,
                    field_kind: Json(FieldKind::Text { is_required: false }),
                    created_at: now,
                    updated_at: None,
                },
            ],
            entries,
            children: Vec::new(),
        };

        // Write the data
        let mut spreadsheet = new_file_empty_worksheet();
        crate::io::export_table_to_excel(&mut spreadsheet, table_data);

        let path = std::path::Path::new("./testing/export_test.xlsx");
        writer::xlsx::write(&spreadsheet, path).anyhow()

        // There is no longer any way to reliabily verify that *specifically* the writer worked
        // without manually checking. So if it wrote out the file, go check it.
    }

    #[test]
    fn export_table_to_csv() -> Result<()> {
        // Base data gen
        let now = Utc::now();
        let table_id = 123;

        let mut entries1 = HashMap::<i32, Cell>::new();
        entries1.insert(1, Cell::String("help2".to_string()));
        entries1.insert(0, Cell::String("help1".to_string()));

        let mut entries2 = HashMap::<i32, Cell>::new();
        entries2.insert(1, Cell::String("help4".to_string()));
        entries2.insert(0, Cell::String("help3".to_string()));

        let table_data = TableData {
            table: Table {
                table_id,
                name: "Dummy Table".to_string(),
                description: "Exported to CSV.".to_string(),
                parent_id: None,
                created_at: now,
                updated_at: None,
            },
            fields: vec![
                Field {
                    field_id: 0,
                    name: "Field 1".into(),
                    table_id,
                    ordering: 0,
                    field_kind: Json(FieldKind::Text { is_required: false }),
                    created_at: now,
                    updated_at: None,
                },
                Field {
                    field_id: 1,
                    name: "Field 2".into(),
                    table_id,
                    ordering: 0,
                    field_kind: Json(FieldKind::Text { is_required: false }),
                    created_at: now,
                    updated_at: None,
                },
            ],
            entries: vec![
                Entry {
                    entry_id: 0,
                    parent_id: None,
                    created_at: now,
                    updated_at: None,
                    cells: entries1,
                },
                Entry {
                    entry_id: 1,
                    parent_id: None,
                    created_at: now,
                    updated_at: None,
                    cells: entries2,
                },
            ],
            children: Vec::new(),
        };

        let p = format!("./testing/export_test-{now}.csv");
        let path = std::path::Path::new(&p);
        let writer = csv::Writer::from_path(path)?;
        crate::io::export_table_to_csv(writer, table_data).anyhow()

        // There is no longer any way to reliabily verify that *specifically* the writer worked
        // without manually checking. So if it wrote out the file, go check it.
    }
}

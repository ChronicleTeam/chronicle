import { GET, POST, PATCH, DELETE, hydrateJSONTableData, POST_FORM } from "./base.js";
import { type Table, type TableData, type Field, type Entry, } from "../types";

//
// Data Management
//

// Table methods
export const getTables = async (): Promise<Table[]> => GET<Table[]>("/tables");

export const getTableChildren = async (table: Table): Promise<Table[]> => GET<Table[]>(`/tables/${table.table_id}/children`);

export const postCreateTable = async (table: Table): Promise<Table> => POST<Table>("/tables", {
  parent_id: table.parent_id,
  name: table.name,
  description: table.description,
});

export const postImportTable = async (table: File): Promise<Table> => {
  let form = new FormData();

  form.append("file", table)

  if (table.type === "text/csv") {
    return POST_FORM<Table>("/tables/csv", form);
  } else {
    return POST_FORM<Table>("/tables/excel", form)
  }
}

export const getExportTable = async (table: Table, type: "csv" | "excel"): Promise<Blob> => GET<Blob>(`/tables/${table.table_id}/${type}`);


export const patchTable = async (table: Table): Promise<Table> => PATCH<Table>(`/tables/${table.table_id}`, {
  name: table.name,
  description: table.description
});

export const deleteTable = async (table: Table): Promise<void> => DELETE(`/tables/${table.table_id}`);

// Field methods
export const getFields = async (table: Table): Promise<Field[]> => GET<Field[]>(`/tables/${table.table_id}/fields`)
  .then(json => hydrateJSONTableData({ table: { table_id: -1, name: "", user_id: -1, description: "", created_at: new Date() }, fields: json, entries: [], children: [] }).fields)

export const postField = async (field: Field): Promise<Field> => POST<Field>(`/tables/${field.table_id}/fields`, {
  name: field.name,
  field_kind: field.field_kind
});

export const patchField = async (field: Field): Promise<Field> => PATCH<Field>(`/tables/${field.table_id}/fields/${field.field_id}`, {
  name: field.name,
  field_kind: field.field_kind
});

export const deleteField = async (field: Field): Promise<void> => DELETE(`/tables/${field.table_id}/fields/${field.field_id}`);

// Entry methods
export const getTableData = async (table: Table): Promise<TableData> => GET<TableData>(`/tables/${table.table_id}/data`).then(hydrateJSONTableData);

export const postEntries = async (table: Table, entries: Entry[]): Promise<Entry[]> => POST<Entry[]>(`/tables/${table.table_id}/entries`, { parent_id: entries[0].parent_id, entries: entries.map(e => e.cells) });

export const patchEntry = async (table: Table, entry: Entry): Promise<Entry> => PATCH<Entry>(`/tables/${table.table_id}/entries/${entry.entry_id}`, { cells: entry.cells });

export const deleteEntry = async (table: Table, entry: Entry): Promise<void> => DELETE(`/tables/${table.table_id}/entries/${entry.entry_id}`);


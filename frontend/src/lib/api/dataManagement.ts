import { GET, POST, PATCH, DELETE, hydrateJSONTableData } from "./base.js";
import { type Table, type TableData, type Field, type Entry, } from "../types.d.js";

//
// Data Management
//

// Table methods
export const getTables = async (): Promise<Table[]> => GET<Table[]>("/tables");

export const getTableChildren = async (table: Table): Promise<Table[]> => GET<Table[]>(`/tables/${table.table_id}/children`);

export const postTable = async (table: Table): Promise<Table> => POST<Table>("/tables", {
  parent_id: table.parent_id,
  name: table.name,
  description: table.description,
});

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

export const postEntry = async (table: Table, entry: Entry): Promise<Entry> => POST<Entry>(`/tables/${table.table_id}/entries`, { parent_id: entry.parent_id, cells: entry.cells });

export const patchEntry = async (table: Table, entry: Entry): Promise<Entry> => PATCH<Entry>(`/tables/${table.table_id}/entries/${entry.entry_id}`, { cells: entry.cells });

export const deleteEntry = async (table: Table, entry: Entry): Promise<void> => DELETE(`/tables/${table.table_id}/entries/${entry.entry_id}`);


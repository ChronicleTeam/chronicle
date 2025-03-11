import { GET, POST, PUT, DELETE, hydrateJSONDataTable} from "./base.js";
import { type Table, type DataTable, type Field, type Entry, type DateTimeKind, FieldType } from "../types.d.js";

//
// Data Management
//

// Table methods
export const getTables = async (): Promise<Table[]> => GET<Table[]>("/tables");

export const postTable = async (name: string): Promise<Table> => POST<Table>("/tables", {
  name,
  description: "",
});

export const putTable = async (table: Table): Promise<Table> => PUT<Table>(`/tables/${table.table_id}`, {
  name: table.name,
  description: table.description
});

export const deleteTable = async (table: Table): Promise<void> => DELETE(`/tables/${table.table_id}`);

// Field methods
export const getFields = async (table: Table): Promise<Field[]> => GET<Field[]>(`/tables/${table.table_id}/fields`)
  .then(json => hydrateJSONDataTable({ table: { table_id: -1, name: "", user_id: -1, description: "", created_at: new Date() }, fields: json, entries: [] }).fields)

export const postField = async (field: Field): Promise<Field> => POST<Field>(`/tables/${field.table_id}/fields`, {
  name: field.name,
  field_kind: field.field_kind
});

export const putField = async (field: Field): Promise<Field> => PUT<Field>(`/tables/${field.table_id}/fields/${field.field_id}`, {
  name: field.name,
  field_kind: field.field_kind
});

export const deleteField = async (field: Field): Promise<void> => DELETE(`/tables/${field.table_id}/fields/${field.field_id}`);

// Entry methods
export const getDataTable = async (table: Table): Promise<DataTable> => GET<DataTable>(`/tables/${table.table_id}/data`).then(hydrateJSONDataTable);

export const postEntry = async (table: Table, entry: Entry): Promise<Entry> => POST<Entry>(`/tables/${table.table_id}/entries`, entry.cells);

export const putEntry = async (table: Table, entry: Entry): Promise<Entry> => PUT<Entry>(`/tables/${table.table_id}/entries/${entry.entry_id}`, entry.cells);

export const deleteEntry = async (table: Table, entry: Entry): Promise<void> => DELETE(`/tables/${table.table_id}/entries/${entry.entry_id}`);


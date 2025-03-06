import { type Table, type DataTable, type Field, type DateTimeKind, FieldType } from "./types.d.js";

export const API_URL = "http://localhost:3000/api";

//
// API functions
//

// Constants
const httpStatus = {
  OK: 200,

  Unprocessable: 422,

  InternalServerError: 500
};

// types
export type APIError = {
  status: number;
  body: string | {
    [key: string]: string;
  };
};

// Method shortcuts
const GET = async <T,>(endpoint: string): Promise<T> => fetch(API_URL + endpoint).then(handleResponse<T>);

const POST = async <T,>(endpoint: string, jsonBody: any): Promise<T> => fetch(API_URL + endpoint, {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify(jsonBody)
}).then(handleResponse<T>);

const PUT = async <T,>(endpoint: string, jsonBody: any): Promise<T> => fetch(API_URL + endpoint, {
  method: "PUT",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify(jsonBody)
}).then(handleResponse<T>);

const DELETE = async (endpoint: string): Promise<void> => fetch(API_URL + endpoint, {
  method: "DELETE",
}).then(response => {
  if (response.ok) {
    return
  } else {
    throw {
      status: response.status,
      message: response.statusText
    }
  }
});

// Helper methods
const handleResponse = async <T,>(response: Response): Promise<T> => {
  if (response.ok) {
    return await response.json();
  } else {
    throw {
      status: response.status,
      body: await (response.headers.get("Content-Type") === "application/json" ? response.json() : response.text())
        .catch((e) => response.statusText),
    } as APIError
  }
};

type JSONDateTimeKind = DateTimeKind & {
  range_start: string;
  range_end: string;
}

export const hydrateJSONDataTable = (jsonObj: DataTable): DataTable => {
  let outTable = jsonObj;

  for (let i = 0; i < outTable.fields.length; i++) {
    if (outTable.fields[i].field_kind.type === FieldType.DateTime) {
      if ((outTable.fields[i].field_kind as DateTimeKind).range_start !== undefined) {
        (outTable.fields[i].field_kind as DateTimeKind).range_start = new Date((outTable.fields[i].field_kind as JSONDateTimeKind).range_start)
      }

      if ((outTable.fields[i].field_kind as DateTimeKind).range_end !== undefined) {
        (outTable.fields[i].field_kind as DateTimeKind).range_end = new Date((outTable.fields[i].field_kind as JSONDateTimeKind).range_end)
      }

      for (let j = 0; j < outTable.entries.length; j++) {
        outTable.entries[j].cells[outTable.fields[i].field_id] = new Date(outTable.entries[j].cells[outTable.fields[i].field_id] as string)
      }
    }
  }

  return outTable;
}
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
export const getFields = async (table: Table): Promise<Field[]> => GET<Field[]>(`/tables/${table.table_id}/fields`).then(json => hydrateJSONDataTable({ table: { table_id: -1, name: "", user_id: -1, description: "", created_at: new Date() }, fields: json, entries: [] }).fields)

export const postField = async (field: Field): Promise<Field> => POST<Field>(`/tables/${field.table_id}/fields`, {
  name: field.name,
  field_kind: field.field_kind
});

export const putField = async (field: Field): Promise<Field> => PUT<Field>(`/tables/${field.table_id}/fields/${field.field_id}`, {
  name: field.name,
  field_kind: field.field_kind
});

export const deleteField = async (field: Field): Promise<void> => DELETE(`/tables/${field.table_id}/fields/${field.field_id}`);

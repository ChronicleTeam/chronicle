import { goto } from "$app/navigation";
import { env } from "$env/dynamic/public";
import { clearUser, user } from "$lib/user.svelte.js";
import { type Table, type TableData, type Field, type Entry, type DateTimeKind, FieldType } from "../types.d.js";

const API_URL = `${env.PUBLIC_API_URL}/api`;

//
// General resources
//

// types
export type APIError = {
  status: number;
  body: string | {
    [key: string]: string;
  };
};

// Method shortcuts
export const GET = async <T,>(endpoint: string): Promise<T> => fetch(API_URL + endpoint, {
  method: "GET",
  credentials: "include"
}).then(handleResponse<T>);

export const POST = async <T,>(endpoint: string, jsonBody: any): Promise<T> => fetch(API_URL + endpoint, {
  method: "POST",
  credentials: "include",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify(jsonBody)
}).then(handleResponse<T>);

export const POST_FORM = async <T,>(endpoint: string, form: FormData): Promise<T> => fetch(API_URL + endpoint, {
  method: "POST",
  credentials: "include",
  // @ts-ignore (URLSearchParams works fine with FormData as stated in https://developer.mozilla.org/en-US/docs/Web/API/FormData)
  body: form.values().some(v => v instanceof Blob) ? form : new URLSearchParams(form)
}).then(handleResponse<T>);

export const PUT = async <T,>(endpoint: string, jsonBody: any): Promise<T> => fetch(API_URL + endpoint, {
  method: "PUT",
  credentials: "include",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify(jsonBody)
}).then(handleResponse<T>);

export const PATCH = async <T,>(endpoint: string, jsonBody: any): Promise<T> => fetch(API_URL + endpoint, {
  method: "PATCH",
  credentials: "include",
  headers: {
    "Content-Type": "application/json",
  },
  body: JSON.stringify(jsonBody)
}).then(handleResponse<T>);

export const DELETE = async (endpoint: string): Promise<void> => fetch(API_URL + endpoint, {
  method: "DELETE",
  credentials: "include",
}).then(response => {
  if (response.ok) {
    return
  } else {
    throw {
      status: response.status,
      body: response.statusText
    } as APIError
  }
});

// Helper methods
const handleResponse = async <T,>(response: Response): Promise<T> => {
  if (response.ok) {
    if (response.headers.get("Content-Type") === "application/octet-stream") {
      return await response.blob() as T
    } else {
      return await response.json().catch(() => { });
    }
  } else if (response.status === 401) {
    //if unauthorized, redirect to login
    clearUser()
    goto(`/`);
  }

  let err = {
    status: response.status,
    body: await (response.headers.get("Content-Type") === "application/json" ? response.json() : response.text())
      .catch((e) => response.statusText),
  } as APIError

  if (typeof err.body === "object") err.body.toString = () => Object.entries(err.body).filter(e => e[0] !== "toString").map((e) => `${e[0]}: ${e[1]}`).join("\n");
  throw err
};

type JSONDateTimeKind = DateTimeKind & {
  range_start: string;
  range_end: string;
}

export const hydrateJSONTableData = (jsonObj: TableData): TableData => {
  let outTable = jsonObj;

  for (let i = 0; i < outTable.fields.length; i++) {
    if (outTable.fields[i].field_kind.type === FieldType.DateTime) {
      if ((outTable.fields[i].field_kind as DateTimeKind).range_start !== null && (outTable.fields[i].field_kind as DateTimeKind).range_start !== undefined) {
        (outTable.fields[i].field_kind as DateTimeKind).range_start = new Date((outTable.fields[i].field_kind as JSONDateTimeKind).range_start)
      }

      if ((outTable.fields[i].field_kind as DateTimeKind).range_end !== null && (outTable.fields[i].field_kind as DateTimeKind).range_end !== undefined) {
        (outTable.fields[i].field_kind as DateTimeKind).range_end = new Date((outTable.fields[i].field_kind as JSONDateTimeKind).range_end)
      }

      for (let j = 0; j < outTable.entries.length; j++) {
        outTable.entries[j].cells[outTable.fields[i].field_id] = new Date(outTable.entries[j].cells[outTable.fields[i].field_id] as string)
      }
    }
  }

  return outTable;
}


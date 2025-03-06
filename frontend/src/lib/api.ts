import type { Table } from "./types.d.js";

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
  message?: string
  details?: {
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
      message: response.statusText,
      details: response.status === httpStatus.Unprocessable ? await response.json() : undefined
    } as APIError
  }
};

// Table functions
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

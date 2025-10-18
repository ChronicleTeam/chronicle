import { describe, it, expect, vi, beforeEach } from "vitest";

// --- mock base functions 
vi.mock("../../../../src/lib/api/base", () => ({
  GET: vi.fn(),
  POST: vi.fn(),
  PATCH: vi.fn(),
  DELETE: vi.fn(),
  POST_FORM: vi.fn(),
  hydrateJSONTableData: vi.fn()
}));

vi.mock("$env/static/public", () => ({
  PUBLIC_API_URL: "example.com/api"
}));

import {
  getTables,
  getTableChildren,
  postCreateTable,
  postImportTable,
  postExportTable,
  patchTable,
  deleteTable,
  getFields,
  postField,
  patchField,
  deleteField,
  getTableData,
  postEntries,
  patchEntry,
  deleteEntry
} from "../../../../src/lib/api";

import {
  GET,
  POST,
  PATCH,
  DELETE,
  POST_FORM,
  hydrateJSONTableData
} from "../../../../src/lib/api/base";

const table = { table_id: 1, parent_id: null, name: "MyTable", description: "desc" };
const field = { field_id: 11, table_id: 1, name: "f1", field_kind: "string" };
const entry = { entry_id: 101, parent_id: 1, cells: { f1: "val" } };

beforeEach(() => {
  vi.clearAllMocks();
});

//
// TABLE METHODS
//
describe("getTables", () => {
  it("calls GET with /tables", async () => {
    (GET as any).mockResolvedValueOnce([table]);
    const res = await getTables();
    expect(GET).toHaveBeenCalledWith("/tables");
    expect(res).toEqual([table]);
  });
});

describe("getTableChildren", () => {
  it("calls GET with table id", async () => {
    (GET as any).mockResolvedValueOnce([table]);
    const res = await getTableChildren(table as any);
    expect(GET).toHaveBeenCalledWith(`/tables/${table.table_id}/children`);
    expect(res).toEqual([table]);
  });
});

describe("postCreateTable", () => {
  it("calls POST with payload", async () => {
    (POST as any).mockResolvedValueOnce(table);
    const res = await postCreateTable(table as any);
    expect(POST).toHaveBeenCalledWith("/tables", {
      parent_id: null,
      name: "MyTable",
      description: "desc"
    });
    expect(res).toEqual(table);
  });
});

describe("postImportTable", () => {
  it("routes to csv endpoint", async () => {
    const file = new File(["a,b"], "f.csv", { type: "text/csv" });
    (POST_FORM as any).mockResolvedValueOnce(table);
    const res = await postImportTable(file as any);
    expect(POST_FORM).toHaveBeenCalledWith("/tables/csv", expect.any(FormData));
    expect(res).toEqual(table);
  });

  it("routes to excel endpoint", async () => {
    const file = new File([""], "f.xlsx", {
      type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    });
    (POST_FORM as any).mockResolvedValueOnce(table);
    await postImportTable(file as any);
    expect(POST_FORM).toHaveBeenCalledWith("/tables/excel", expect.any(FormData));
  });

  it("throws for unsupported type", async () => {
    const file = new File([""], "f.txt", { type: "text/plain" });
    await expect(postImportTable(file as any)).rejects.toEqual({ body: "Unsupported format" });
  });
});

describe("postExportTable", () => {
  it("calls POST_FORM with correct endpoint", async () => {
    (POST_FORM as any).mockResolvedValueOnce(new Blob(["data"]));
    const res = await postExportTable(table as any, "csv");
    expect(POST_FORM).toHaveBeenCalledWith(`/tables/${table.table_id}/csv`, expect.any(FormData));
    expect(res).toBeInstanceOf(Blob);
  });
});

describe("patchTable", () => {
  it("calls PATCH with table id", async () => {
    (PATCH as any).mockResolvedValueOnce(table);
    const res = await patchTable(table as any);
    expect(PATCH).toHaveBeenCalledWith(`/tables/${table.table_id}`, {
      name: "MyTable",
      description: "desc"
    });
    expect(res).toEqual(table);
  });
});

describe("deleteTable", () => {
  it("calls DELETE with table id", async () => {
    (DELETE as any).mockResolvedValueOnce(undefined);
    await deleteTable(table as any);
    expect(DELETE).toHaveBeenCalledWith(`/tables/${table.table_id}`);
  });
});

//
// FIELD METHODS
//
describe("getFields", () => {
  it("calls GET and hydrates data", async () => {
    (GET as any).mockResolvedValueOnce([field]);
    (hydrateJSONTableData as any).mockReturnValueOnce({ fields: [field] });
    const res = await getFields(table as any);
    expect(GET).toHaveBeenCalledWith(`/tables/${table.table_id}/fields`);
    expect(hydrateJSONTableData).toHaveBeenCalled();
    expect(res).toEqual([field]);
  });
});

describe("postField", () => {
  it("calls POST with field payload", async () => {
    (POST as any).mockResolvedValueOnce(field);
    const res = await postField(field as any);
    expect(POST).toHaveBeenCalledWith(`/tables/${field.table_id}/fields`, {
      name: "f1",
      field_kind: "string"
    });
    expect(res).toEqual(field);
  });
});

describe("patchField", () => {
  it("calls PATCH with field id", async () => {
    (PATCH as any).mockResolvedValueOnce(field);
    const res = await patchField(field as any);
    expect(PATCH).toHaveBeenCalledWith(
      `/tables/${field.table_id}/fields/${field.field_id}`,
      { name: "f1", field_kind: "string" }
    );
    expect(res).toEqual(field);
  });
});

describe("deleteField", () => {
  it("calls DELETE with field id", async () => {
    (DELETE as any).mockResolvedValueOnce(undefined);
    await deleteField(field as any);
    expect(DELETE).toHaveBeenCalledWith(`/tables/${field.table_id}/fields/${field.field_id}`);
  });
});

//
//  ENTRY METHODS
//
describe("getTableData", () => {
  it("calls GET and hydrates data", async () => {
    const tableData = { fields: [field], entries: [entry] };
    (GET as any).mockResolvedValueOnce(tableData);
    (hydrateJSONTableData as any).mockReturnValueOnce(tableData);
    const res = await getTableData(table.table_id.toString());
    expect(GET).toHaveBeenCalledWith(`/tables/${table.table_id}/data`);
    expect(hydrateJSONTableData).toHaveBeenCalledWith(tableData);
    expect(res).toEqual(tableData);
  });
});

describe("postEntries", () => {
  it("calls POST with mapped entries", async () => {
    (POST as any).mockResolvedValueOnce([entry]);
    const res = await postEntries(table as any, [entry] as any);
    expect(POST).toHaveBeenCalledWith(`/tables/${table.table_id}/entries`, {
      parent_id: 1,
      entries: [{ f1: "val" }]
    });
    expect(res).toEqual([entry]);
  });
});

describe("patchEntry", () => {
  it("calls PATCH with entry id", async () => {
    (PATCH as any).mockResolvedValueOnce(entry);
    const res = await patchEntry(table as any, entry as any);
    expect(PATCH).toHaveBeenCalledWith(
      `/tables/${table.table_id}/entries/${entry.entry_id}`,
      { cells: { f1: "val" } }
    );
    expect(res).toEqual(entry);
  });
});

describe("deleteEntry", () => {
  it("calls DELETE with entry id", async () => {
    (DELETE as any).mockResolvedValueOnce(undefined);
    await deleteEntry(table as any, entry as any);
    expect(DELETE).toHaveBeenCalledWith(`/tables/${table.table_id}/entries/${entry.entry_id}`);
  });
});

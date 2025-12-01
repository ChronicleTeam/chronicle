import { describe, it, expect, vi, beforeEach } from "vitest";

// ---- mock API base layer
vi.mock("../../../../src/lib/api/base", () => ({
  GET: vi.fn(),
  POST: vi.fn(),
  PATCH: vi.fn(),
  DELETE: vi.fn(),
}));

import { GET, POST, PATCH, DELETE } from "../../../../src/lib/api/base";

import {
  getAllTableAccess,
  getAllDashboardAccess,
  createTableAccess,
  createDashboardAccess,
  deleteTableAccess,
  deleteDashboardAccess,
  patchTableAccess,
  patchDashboardAccess,
} from "../../../../src/lib/api/access";

import type { AccessRole } from "../../../../src/lib/types/access";

const mockAccess = [
  { username: "alice", access_role: "Viewer" as AccessRole },
  { username: "bob", access_role: "Editor" as AccessRole },
];

beforeEach(() => {
  vi.clearAllMocks();
});


// GET ALL ACCESS

describe("getAllTableAccess", () => {
  it("calls GET with /Table/{id}/access and returns access list", async () => {
    (GET as any).mockResolvedValueOnce(mockAccess);

    const res = await getAllTableAccess("123");

    expect(GET).toHaveBeenCalledWith("/Table/123/access");
    expect(res).toEqual(mockAccess);
  });
});

describe("getAllDashboardAccess", () => {
  it("calls GET with /Dashboard/{id}/access", async () => {
    (GET as any).mockResolvedValueOnce(mockAccess);

    const res = await getAllDashboardAccess("99");

    expect(GET).toHaveBeenCalledWith("/Dashboard/99/access");
    expect(res).toEqual(mockAccess);
  });
});


// CREATE ACCESS

describe("createTableAccess", () => {
  it("calls POST with /Table/{id}/access and body", async () => {
    const body = {
      username: "charlie",
      access_role: "Owner" as AccessRole,
    };

    (POST as any).mockResolvedValueOnce(undefined);

    await createTableAccess("55", body);

    expect(POST).toHaveBeenCalledWith("/Table/55/access", body);
  });
});

describe("createDashboardAccess", () => {
  it("calls POST with /DashboardTable/{id}/access and body", async () => {
    const body = {
      username: "david",
      access_role: "Viewer" as AccessRole,
    };

    (POST as any).mockResolvedValueOnce(undefined);

    await createDashboardAccess("10", body);

    expect(POST).toHaveBeenCalledWith("/DashboardTable/10/access", body);
  });
});


// DELETE ACCESS

describe("deleteTableAccess", () => {
  it("calls DELETE with /Table/{id}/access and username payload array", async () => {
    (DELETE as any).mockResolvedValueOnce(undefined);

    await deleteTableAccess("123", "alice");

    expect(DELETE).toHaveBeenCalledWith("/Table/123/access", [
      { username: "alice" },
    ]);
  });
});

describe("deleteDashboardAccess", () => {
  it("calls DELETE with /Dashboard/{id}/access and username payload", async () => {
    (DELETE as any).mockResolvedValueOnce(undefined);

    await deleteDashboardAccess("88", "bob");

    expect(DELETE).toHaveBeenCalledWith("/Dashboard/88/access", [
      { username: "bob" },
    ]);
  });
});


// PATCH ACCESS 

describe("patchTableAccess", () => {
  it("calls PATCH with /Table/{id}/access and correct role body", async () => {
    (PATCH as any).mockResolvedValueOnce(undefined);

    await patchTableAccess("77", "john", "Editor" as AccessRole);

    expect(PATCH).toHaveBeenCalledWith("/Table/77/access", [
      { username: "john", access_role: "Editor" },
    ]);
  });
});

describe("patchDashboardAccess", () => {
  it("calls PATCH with /Table/{id}/access (as written in your code!)", async () => {
    (PATCH as any).mockResolvedValueOnce(undefined);

    await patchDashboardAccess("44", "sam", "Owner" as AccessRole);

    
    expect(PATCH).toHaveBeenCalledWith("/Table/44/access", [
      { username: "sam", access_role: "Owner" },
    ]);
  });
});

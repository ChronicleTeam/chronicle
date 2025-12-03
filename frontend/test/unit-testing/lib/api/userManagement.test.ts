import { describe, it, expect, vi, beforeEach } from "vitest";

// ---- mock API base layer
vi.mock("../../../../src/lib/api/base", () => ({
  GET: vi.fn(),
  POST: vi.fn(),
  POST_FORM2: vi.fn(),
  PATCH: vi.fn(),
  PATCH_FORM2: vi.fn(),
  DELETE: vi.fn(),
}));

import { GET, POST, PATCH, DELETE, POST_FORM2, PATCH_FORM2 } from "../../../../src/lib/api/base";

import {
  getAllUsers,
  createUser,
  updateUser,
  deleteUser,
  type UserResponse,
  type CreateUser,
  type UpdateUser
} from "../../../../src/lib/api/userManagement";

const mockUsers: UserResponse[] = [
  { user_id: 1, username: "alice", is_admin: true },
  { user_id: 2, username: "bob", is_admin: false },
];

beforeEach(() => {
  vi.clearAllMocks();
});

// ------------------- GET ALL USERS -------------------
describe("getAllUsers", () => {
  it("calls GET with /api/users and returns user list", async () => {
    (GET as any).mockResolvedValueOnce(mockUsers);

    const res = await getAllUsers();

    expect(GET).toHaveBeenCalledWith("/users");
    expect(res).toEqual(mockUsers);
  });
});

// ------------------- CREATE USER -------------------
describe("createUser", () => {
  it("calls POST with /api/users and body", async () => {
    const body: CreateUser = { username: "charlie", password: "secret123" };
    const urlParams: URLSearchParams = new URLSearchParams();
    urlParams.append("username", "charlie");
    urlParams.append("password", "secret123");

    (POST as any).mockResolvedValueOnce(undefined);

    await createUser(body);

    expect(POST_FORM2).toHaveBeenCalledWith("/users", urlParams);
  });
});

// ------------------- UPDATE USER -------------------
describe("updateUser", () => {
  it("calls PATCH with /api/users/{id} and body", async () => {
    const body: UpdateUser = { username: "bob2", password: "newpass" };
    const urlParams: URLSearchParams = new URLSearchParams();
    urlParams.append("username", "bob2");
    urlParams.append("password", "newpass");

    (PATCH as any).mockResolvedValueOnce(undefined);

    await updateUser(2, body);

    expect(PATCH_FORM2).toHaveBeenCalledWith("/users/2", urlParams);
  });

  it("can handle partial update (only username)", async () => {
    const body: UpdateUser = { username: "bob3" };
    const urlParams: URLSearchParams = new URLSearchParams();
    urlParams.append("username", "bob3");

    (PATCH as any).mockResolvedValueOnce(undefined);

    await updateUser(2, body);

    expect(PATCH_FORM2).toHaveBeenCalledWith("/users/2", urlParams);
  });
});

// ------------------- DELETE USER -------------------
describe("deleteUser", () => {
  it("calls DELETE with /api/users/{id}", async () => {
    (DELETE as any).mockResolvedValueOnce(undefined);

    await deleteUser(1);

    expect(DELETE).toHaveBeenCalledWith("/users/1");
  });
});

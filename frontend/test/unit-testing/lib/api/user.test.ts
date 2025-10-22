import { describe, it, expect, vi, beforeEach } from "vitest";

// --- mock base functions
vi.mock("../../../../src/lib/api/base", () => ({
  GET: vi.fn(),
  POST: vi.fn(),
  POST_FORM: vi.fn()
}));

import { GET, POST_FORM } from "../../../../src/lib/api/base";
import { postLogin, getLogout, getUser } from "../../../../src/lib/api/user";

const mockUser = { id: 1, username: "john_doe", email: "john@example.com" };

beforeEach(() => {
  vi.clearAllMocks();
});

//
// postLogin
//
describe("postLogin", () => {
  it("calls POST_FORM with /login and credentials", async () => {
    const credentials = { username: "john_doe", password: "secret" };
    (POST_FORM as any).mockResolvedValueOnce(mockUser);

    const res = await postLogin(credentials as any);

    expect(POST_FORM).toHaveBeenCalledWith("/login", expect.any(FormData));

    // check that FormData contains username & password
    const callArgs = (POST_FORM as any).mock.calls[0][1] as FormData;
    expect(callArgs.get("username")).toBe("john_doe");
    expect(callArgs.get("password")).toBe("secret");

    expect(res).toEqual(mockUser);
  });
});

//
// getLogout
//
describe("getLogout", () => {
  it("calls GET with /logout", async () => {
    (GET as any).mockResolvedValueOnce(undefined);
    await getLogout();
    expect(GET).toHaveBeenCalledWith("/logout");
  });
});

//
// getUser
//
describe("getUser", () => {
  it("calls GET with /user and returns user", async () => {
    (GET as any).mockResolvedValueOnce(mockUser);
    const res = await getUser();
    expect(GET).toHaveBeenCalledWith("/user");
    expect(res).toEqual(mockUser);
  });
});

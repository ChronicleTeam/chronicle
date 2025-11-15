export type AccessRole = "Owner" | "Editor" | "Viewer";

export type Access = {
  access_role: AccessRole;
  username: string;
}

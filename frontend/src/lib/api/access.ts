import { GET, POST, DELETE, PATCH, } from "./base.js";
import { type AccessRole, type Access } from "$lib/types/access.js";

const getAllAccess = async (resource: string, resource_id: string): Promise<Access[]> => GET<Access[]>(`/${resource}/${resource_id}/access`);
export const getAllTableAccess = async (table_id: string): Promise<Access[]> => getAllAccess('Table', table_id);
export const getAllDashboardAccess = async (dashboard_id: string): Promise<Access[]> => getAllAccess('Dashboard', dashboard_id);

const createAccess = async (resource: string, resource_id: string, access: Access): Promise<void> => POST<void>(`/${resource}/${resource_id}/access`, access);
export const createTableAccess = async (table_id: string, access: Access): Promise<void> => createAccess('Table', table_id, access);
export const createDashboardAccess = async (dashboard_id: string, access: Access): Promise<void> => createAccess('DashboardTable', dashboard_id, access);

const deleteAccess = async (resource: string, resource_id: string, username: string): Promise<void> => DELETE(`/${resource}/${resource_id}/access`, [{ username }]);
export const deleteTableAccess = async (table_id: string, username: string): Promise<void> => deleteAccess('Table', table_id, username);
export const deleteDashboardAccess = async (dashboard_id: string, username: string): Promise<void> => deleteAccess('Dashboard', dashboard_id, username);

const patchAccess = async (resource: string, resource_id: string, username: string, role: AccessRole): Promise<void> => PATCH<void>(`/${resource}/${resource_id}/access`, [{ username, access_role: role }]);
export const patchTableAccess = async (table_id: string, username: string, role: AccessRole): Promise<void> => patchAccess('Table', table_id, username, role);
export const patchDashboardAccess = async (table_id: string, username: string, role: AccessRole): Promise<void> => patchAccess('Table', table_id, username, role);


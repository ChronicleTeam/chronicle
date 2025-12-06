import { getTableData, getAllTableAccess, getAllUsers } from "$lib/api";
import type { PageLoad } from './$types';

export const ssr = false;



export const load: PageLoad = async ({ params }) => {
  // fetch table and sort it
  const tableResponse = await getTableData(params.table_id)
  tableResponse.table_data.fields.sort((f, g) => f.ordering - g.ordering);

  // fetch all users and all users with access, if possible
  const users = await getAllUsers().catch(() => undefined);
  const allAccess = await getAllTableAccess(params.table_id).catch(() => undefined);
  return {
    table: tableResponse.table_data,
    role: tableResponse.access_role,
    users,
    allAccess,
  }
}

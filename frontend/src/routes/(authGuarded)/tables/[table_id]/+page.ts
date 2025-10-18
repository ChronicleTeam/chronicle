import { getTableData } from "$lib/api";
import type { TableData } from "$lib/types";
import type { PageLoad } from './$types';

export const ssr = false;



export const load: PageLoad = async ({ params }) => {
  const tableResponse = await getTableData(params.table_id)
  tableResponse.table_data.fields.sort((f, g) => f.ordering - g.ordering);
  return {
    table: tableResponse.table_data,
    role: tableResponse.access_role
  }
}

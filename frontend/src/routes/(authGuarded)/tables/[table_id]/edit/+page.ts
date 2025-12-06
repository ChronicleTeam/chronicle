import { getTableData } from "$lib/api";
import type { PageLoad } from './$types';

export const ssr = false;



// load table data
export const load: PageLoad = async ({ params }) => {
  return {
    table_prop: await getTableData(params.table_id).then((response) => {
      return response.table_data.table;
    })
  }
}


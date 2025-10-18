import { getTableData } from "$lib/api";
import type { TableData } from "$lib/types";
import type { PageLoad } from './$types';

export const ssr = false;



export const load: PageLoad = async ({ params }) => {
  return {
    entryId: params.entry_id,
    table: await getTableData(params.subtable_id).then((response: TableData) => {
      response.fields.sort((f, g) => f.ordering - g.ordering);

      return response;
    })
  }
}


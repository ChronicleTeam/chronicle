import { getTableData, getTables } from "$lib/api";
import type { LayoutLoad } from './$types';

export const ssr = false;



export const load: LayoutLoad = async ({ params }) => {
  let subtable;
  if (params.subtable_id) {
    subtable = await getTableData(params.subtable_id);
  }
  return {
    tables: await getTables(),
    subtable
  }
}


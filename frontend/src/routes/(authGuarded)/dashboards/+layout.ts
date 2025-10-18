import { getDashboards } from "$lib/api";
import type { LayoutLoad } from './$types';

export const ssr = false;



export const load: LayoutLoad = async () => {
  return {
    dashboards: await getDashboards(),
  }
}



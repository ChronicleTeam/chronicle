import { getCharts, getTables, getDashboards } from "$lib/api";
import type { Dashboard } from "$lib/types";
import type { PageLoad } from './$types';

export const ssr = false;



export const load: PageLoad = async ({ params }) => {
  return {
    tables: await getTables(),
    dashboard: (await getDashboards()).find((dashboard) => dashboard.dashboard_id.toString() === params.dashboard_id) as Dashboard,
    charts: await getCharts(params.dashboard_id)
  }
}


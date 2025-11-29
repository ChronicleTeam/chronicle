import { getCharts, getTables, getDashboards, getAllDashboardAccess, getAllUsers } from "$lib/api";
import type { Dashboard } from "$lib/types";
import type { PageLoad } from './$types';

export const ssr = false;



export const load: PageLoad = async ({ params }) => {
  const users = await getAllUsers().catch(() => null);
  const allAccess = await getAllDashboardAccess(params.dashboard_id).catch(() => null);
  return {
    tables: await getTables(),
    dashboard: (await getDashboards()).map((dashBoardItem) => dashBoardItem.dashboard).find((dashboard) => dashboard.dashboard_id.toString() === params.dashboard_id) as Dashboard,
    charts: await getCharts(params.dashboard_id),
    users,
    allAccess
  }
}


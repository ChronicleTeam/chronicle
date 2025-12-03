import { getChartData, getDashboards, getTableData } from "$lib/api";
import type { Dashboard } from "$lib/types";
import type { PageLoad } from './$types';

export const ssr = false;



export const load: PageLoad = async ({ params }) => {
  const chartData = await getChartData(params.dashboard_id, params.chart_id);
  return {
    dashboard: (await getDashboards()).find((dashboard) => dashboard.dashboard.dashboard_id.toString() === params.dashboard_id)?.dashboard as Dashboard,
    chartData,
    tableData: await getTableData(chartData.chart.table_id.toString())
  }
}



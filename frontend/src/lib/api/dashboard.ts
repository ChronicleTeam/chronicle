import { GET, POST, PUT, DELETE, PATCH, } from "./base.js";
import { type Dashboard, type Chart, type ChartData, type AxisField, type Axis, type Cells, FieldType } from "../types.d.js"

// Dashboard methods

export const getDashboards = async (): Promise<Dashboard[]> => GET<Dashboard[]>("/dashboards");

export const postDashboard = async (name: string): Promise<Dashboard> => POST<Dashboard>("/dashboards", {
  name,
  description: ""
});

export const patchDashboard = async (d: Dashboard): Promise<Dashboard> => PATCH<Dashboard>(`/dashboards/${d.dashboard_id}`, {
  name: d.name,
  description: d.description
});

export const deleteDashboard = async (d: Dashboard): Promise<void> => DELETE(`/dashboards/${d.dashboard_id}`);

// Chart methods

export const getCharts = async (d: Dashboard): Promise<Chart[]> => GET<Chart[]>(`/dashboards/${d.dashboard_id}/charts`);

export const getChartData = async (d: Dashboard, c: Chart): Promise<ChartData> => GET<ChartData>(`/dashboards/${d.dashboard_id}/charts/${c.chart_id}/data`).then((c: ChartData) => {
  c.axes = c.axes.map((a: AxisField) => {
    a.axis.aggregate = a.axis.aggregate ?? undefined;
    return a;
  });

  c.axes.forEach((a: AxisField) => {
    if (a.field_kind.type === FieldType.DateTime) {
      c.cells = c.cells.map((row: Cells) => {
        row[a.axis.axis_id] = (new Date(row[a.axis.axis_id] as string));
        return row;
      })
    }
  });

  return c;
});

export const postChart = async (d: Dashboard, c: Chart): Promise<Chart> => POST<Chart>(`/dashboards/${d.dashboard_id}/charts`, {
  table_id: c.table_id,
  name: c.name,
  chart_kind: c.chart_kind
});

export const patchChart = async (d: Dashboard, c: Chart): Promise<Chart> => PATCH<Chart>(`/dashboards/${d.dashboard_id}/charts/${c.chart_id}`, {
  name: c.name,
  chart_kind: c.chart_kind
});

export const deleteChart = async (d: Dashboard, c: Chart): Promise<void> => DELETE(`/dashboards/${d.dashboard_id}/charts/${c.chart_id}`);

// Axis methods
export const putAxes = async (d: Dashboard, c: Chart, axes: Axis[]): Promise<Axis[]> => PUT(`/dashboards/${d.dashboard_id}/charts/${c.chart_id}/axes`, axes.map(a => ({
  field_id: a.field_id,
  axis_kind: a.axis_kind,
  aggregate: a.aggregate ?? undefined
})));

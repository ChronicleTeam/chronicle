import { describe, it, expect, vi, beforeEach } from "vitest";

// mock base functions 
vi.mock("../../../../src/lib/api/base", () => ({
  GET: vi.fn(),
  POST: vi.fn(),
  PATCH: vi.fn(),
  PUT: vi.fn(),
  DELETE: vi.fn(),
  _TESTING: { handleResponse: vi.fn() }
}));

vi.mock("$env/static/public", () => ({
  PUBLIC_API_URL: "example.com/api"
}));
vi.mock("$lib/user.svelte.js", () => ({
  clearUser: vi.fn()
}));
vi.mock("$app/navigation", () => ({
  goto: vi.fn()
}));
vi.stubGlobal(
  "fetch",
  vi.fn(() => Promise.resolve(Response.json({ test: "hello" })))
);


import {
  getDashboards,
  postDashboard,
  patchDashboard,
  deleteDashboard,
  getCharts,
  getChartData,
  postChart,
  patchChart,
  deleteChart,
  putAxes
} from "../../../../src/lib/api/dashboard";

import { GET, POST, PATCH, PUT, DELETE, _TESTING } from "../../../../src/lib/api/base";
import { FieldType } from "../../../../src/lib/types/dataManagement";

const handleResponse = _TESTING.handleResponse;

describe("Dashboard & Chart API", () => {
  const dashboard = { dashboard_id: 1, name: "SEG", description: "test" };
  const chart = { chart_id: 10, table_id: 5, name: "Chart1", chart_kind: "bar" };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("getDashboards calls GET with /dashboards", async () => {
    ;(GET as any).mockResolvedValueOnce([{ dashboard_id: 1 }]);


    const res = await getDashboards();
    expect(GET).toHaveBeenCalledWith("/dashboards");
    expect(res).toEqual([{ dashboard_id: 1 }]);
  });


  it("postDashboard calls POST with name and empty description", async () => {
    (POST as any).mockResolvedValueOnce(dashboard);
    const res = await postDashboard("SEG");
    expect(POST).toHaveBeenCalledWith("/dashboards", {
      name: "SEG",
      description: "",
    });
    expect(res).toEqual(dashboard);
  });

  it("patchDashboard calls PATCH with dashboard id", async () => {
    (PATCH as any).mockResolvedValueOnce(dashboard);
    const res = await patchDashboard(dashboard as any);
    expect(PATCH).toHaveBeenCalledWith(`/dashboards/${dashboard.dashboard_id}`, {
      name: "SEG",
      description: "test",
    });
    expect(res).toEqual(dashboard);
  });

  it("deleteDashboard calls DELETE with dashboard id", async () => {
    (DELETE as any).mockResolvedValueOnce(undefined);
    await deleteDashboard(dashboard as any);
    expect(DELETE).toHaveBeenCalledWith(`/dashboards/${dashboard.dashboard_id}`);
  });

  // Chart tests
  it("getCharts calls GET with correct dashboard id", async () => {
    (GET as any).mockResolvedValueOnce([chart]);
    const res = await getCharts(dashboard as any);
    expect(GET).toHaveBeenCalledWith(`/dashboards/${dashboard.dashboard_id}/charts`);
    expect(res).toEqual([chart]);
  });

  it("getChartData transforms aggregate and DateTime fields", async () => {
    const chartData = {
      axes: [
        {
          axis: { axis_id: "x", aggregate: null },
          field_kind: { type: FieldType.DateTime },
        },
      ],
      cells: [{ x: "2025-05-05T23:23:23Z" }],
    };

    (GET as any).mockResolvedValueOnce(chartData);

    const res = await getChartData(dashboard as any, chart as any);

    
    expect(res.axes[0].axis.aggregate).toBeUndefined();

    
    expect(res.cells[0].x).toBeInstanceOf(Date);
  });

  it("postChart calls POST with correct endpoint and payload", async () => {
    (POST as any).mockResolvedValueOnce(chart);
    const res = await postChart(dashboard as any, chart as any);
    expect(POST).toHaveBeenCalledWith(`/dashboards/${dashboard.dashboard_id}/charts`, {
      table_id: 5,
      name: "Chart1",
      chart_kind: "bar",
    });
    expect(res).toEqual(chart);
  });

  it("patchChart calls PATCH with chart id", async () => {
    (PATCH as any).mockResolvedValueOnce(chart);
    const res = await patchChart(dashboard as any, chart as any);
    expect(PATCH).toHaveBeenCalledWith(`/dashboards/${dashboard.dashboard_id}/charts/${chart.chart_id}`, {
      name: "Chart1",
      chart_kind: "bar",
    });
    expect(res).toEqual(chart);
  });

  it("deleteChart calls DELETE with chart id", async () => {
    (DELETE as any).mockResolvedValueOnce(undefined);
    await deleteChart(dashboard as any, chart as any);
    expect(DELETE).toHaveBeenCalledWith(`/dashboards/${dashboard.dashboard_id}/charts/${chart.chart_id}`);
  });

  // Axis 
  it("putAxes calls PUT with mapped axes", async () => {
    const axes = [
      { field_id: 1, axis_kind: "x", aggregate: null },
      { field_id: 2, axis_kind: "y", aggregate: "sum" },
    ];
    (PUT as any).mockResolvedValueOnce(axes);

    await putAxes(dashboard as any, chart as any, axes as any);

    expect(PUT).toHaveBeenCalledWith(
      `/dashboards/${dashboard.dashboard_id}/charts/${chart.chart_id}/axes`,
      [
        { field_id: 1, axis_kind: "x", aggregate: undefined },
        { field_id: 2, axis_kind: "y", aggregate: "sum" },
      ]
    );
  });
});

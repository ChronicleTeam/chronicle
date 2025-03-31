<script lang="ts">
  import {
    deleteChart,
    getChartData,
    getCharts,
    getTableData,
    getTables,
    patchChart,
    patchDashboard,
    postChart,
    putAxes,
  } from "$lib/api";
  import ChartComponent from "$lib/components/charts/Chart.svelte";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import {
    type Dashboard,
    type Chart,
    type Axis,
    ChartKind,
    AxisKind,
    type Table,
    type TableData,
    type AxisField,
    type FieldKind,
    type ChartData,
    Aggregate,
  } from "$lib/types.d.js";
  import { onMount } from "svelte";
  import type { ModeState } from "./types";
  import { EditMode } from "./types";

  let {
    dashboard,
    removeDashboard,
  }: {
    dashboard: Dashboard;
    removeDashboard: () => void;
  } = $props();

  //
  // Constants
  //

  const col_start = [
    "",
    "col-start-1",
    "col-start-2",
    "col-start-3",
    "col-start-4",
    "col-start-5",
    "col-start-6",
    "col-start-7",
    "col-start-8",
  ];

  const col_span = [
    "",
    "col-span-1",
    "col-span-2",
    "col-span-3",
    "col-span-4",
    "col-span-5",
    "col-span-6",
    "col-span-7",
    "col-span-8",
  ];

  const row_start = [
    "",
    "row-start-1",
    "row-start-2",
    "row-start-3",
    "row-start-4",
    "row-start-5",
    "row-start-6",
    "row-start-7",
    "row-start-8",
  ];

  const row_span = [
    "",
    "row-span-1",
    "row-span-2",
    "row-span-3",
    "row-span-4",
    "row-span-5",
    "row-span-6",
    "row-span-7",
    "row-span-8",
  ];

  const asyncTables = $state(getTables());

  //
  // State
  //

  let modeState: ModeState = $state({ mode: EditMode.DISPLAY });
  const modeDisplay = () => {
    modeState = { mode: EditMode.DISPLAY };
  };
  const modeEditDash = () => {
    modeState = {
      mode: EditMode.EDIT_DASH,
      metadataChanged: false,
      newChart: null,
    };
  };
  const modeEditChart = (
    chartIdx: number,
    chartTableData: TableData,
    axisFields: AxisField[],
  ) => {
    modeState = {
      mode: EditMode.EDIT_CHART,
      chartIdx,
      chartTableData,
      axisFields,
    };
  };

  let charts: Chart[] = $state([]);

  let freeSpaces = $derived.by(() => {
    let out = [];
    for (let i = 1; i <= 4; i++) {
      for (let j = 1; j <= 1; j++) {
        if (
          !charts.some((c) => withinChart(i, j, c)) &&
          (!(modeState.mode === EditMode.EDIT_DASH) ||
            !modeState.newChart ||
            !withinChart(i, j, modeState.newChart))
        ) {
          out.push([i, j]);
        }
      }
    }

    return out;
  });

  let errors: {
    dashboard: {
      save: string;
    };

    chart: {
      create: string;
      edit: string;
      save: string;
      load: string;
    };

    axes: {
      save: { [key: string]: string };
    };
  } = $state({
    dashboard: {
      save: "",
    },

    chart: {
      create: "",
      edit: "",
      save: "",
      load: "",
    },

    axes: {
      save: {},
    },
  });

  //
  // Helper methods
  //
  const blankChart = (x: number, y: number, w: number, h: number): Chart => {
    let i = 0;
    while (charts.some((c) => c.name === `Chart ${++i}`));
    return {
      chart_id: -1,
      dashboard_id: dashboard.dashboard_id,
      table_id: -1,
      name: `Chart ${i}`,
      chart_kind: ChartKind.Bar,
      x,
      y,
      w,
      h,
    };
  };

  const blankAxis = (c: Chart): Axis => {
    let kinds = Object.values(AxisKind);
    let i = 0;
    if (modeState.mode === EditMode.EDIT_CHART) {
      while (
        modeState.axisFields.some((af) => af.axis.axis_kind === kinds[i]) &&
        i < kinds.length
      ) {
        i++;
      }
    }

    let j = -1;
    if (modeState.mode === EditMode.EDIT_CHART) {
      while (modeState.axisFields.some((af) => af.axis.axis_id === j)) {
        j--;
      }
    }

    return {
      axis_id: j,
      chart_id: c.chart_id,
      field_id: -1,
      axis_kind: c.chart_kind === ChartKind.Table ? AxisKind.Label : kinds[i],
    };
  };

  const withinChart = (x: number, y: number, c: Chart): boolean =>
    c.x <= x && x < c.x + c.w && c.y <= y && y < c.y + c.h;

  const cancelCreateChart = () => {
    if (modeState.mode === EditMode.EDIT_DASH) {
      modeState.newChart = null;
    }
  };

  const cancelEditChart = () => {
    errors.chart.save = "";
    errors.axes.save = {};
    modeDisplay();
  };

  //
  // API
  //

  const saveDashboard = () => {
    patchDashboard(dashboard)
      .then((r) => {
        dashboard.name = r.name;
        dashboard.description = r.description;
        if (modeState.mode === EditMode.EDIT_DASH) {
          modeState.metadataChanged = false;
        }
        errors.dashboard.save = "";
      })
      .catch((e) => {
        errors.dashboard.save = e.body.toString();
      });
  };

  const loadCharts = () =>
    getCharts(dashboard)
      .then((result: Chart[]) => {
        errors.chart.load = "";
        charts = result.map((c, i) => {
          c.x = i + 1;
          c.y = 1;
          c.w = 1;
          c.h = 1;
          return c;
        });
      })
      .catch((e) => {
        errors.chart.load = e.body.toString();
      });

  const createChart = () => {
    if (modeState.mode === EditMode.EDIT_DASH && modeState.newChart) {
      postChart(dashboard, modeState.newChart)
        .then(loadCharts)
        .then(cancelCreateChart)
        .then(() => {
          errors.chart.create = "";
          modeDisplay();
        })
        .catch((e) => {
          errors.chart.create = e.body.toString();
        });
    }
  };

  const editChart = (c: Chart) => {
    getChartData(dashboard, c)
      .then(async (r) => {
        modeEditChart(
          charts.findIndex((d) => d.chart_id === c.chart_id),
          await getTables().then((t) =>
            getTableData(
              t.find((table) => table.table_id === r.chart.table_id) as Table,
            ).catch(() => {
              throw { body: "Could not get Chart data" };
            }),
          ),
          r.axes,
        );
        errors.chart.edit = "";
        errors.axes.save = {};
      })
      .catch((e) => {
        errors.chart.edit = e.body.toString();
      });
  };

  const removeChart = (c: Chart) => {
    deleteChart(dashboard, c)
      .then(() => {
        loadCharts();
        errors.chart.edit = "";
      })
      .catch((e) => {
        errors.chart.edit = e.body.toString();
      });
  };

  const saveChartWithAxisFields = (chart: Chart, axes: AxisField[]) => {
    let chartPromise = patchChart(dashboard, chart)
      .then(() => {
        errors.chart.save = "";
      })
      .catch((e) => {
        errors.chart.save = e.body.toString();
        throw Error();
      });
    let axisPromise = putAxes(
      dashboard,
      chart,
      axes.map((af) => af.axis),
    )
      .then(() => {
        errors.axes.save = {};
      })
      .catch((e) => {
        errors.axes.save = e.body;
        throw Error();
      });

    Promise.all([chartPromise, axisPromise]).then(() => {
      cancelEditChart();
    });
  };

  //
  // Startup
  //

  onMount(() => {
    loadCharts();
  });
</script>

{#if modeState.mode === EditMode.DISPLAY || modeState.mode === EditMode.EDIT_DASH}
  {#if modeState.mode === EditMode.DISPLAY}
    <div class="flex flex-col items-center">
      <h2 class="font-bold text-xl">{dashboard.name}</h2>
      <p>{dashboard.description}</p>
    </div>
  {:else if modeState.mode === EditMode.EDIT_DASH}
    <div class="flex flex-col items-center">
      <input
        bind:value={() => dashboard.name,
        (s) => {
          if (modeState.mode === EditMode.EDIT_DASH) {
            modeState.metadataChanged = true;
            dashboard.name = s;
          }
        }}
      />
      <input
        bind:value={() => dashboard.description,
        (s) => {
          if (modeState.mode === EditMode.EDIT_DASH) {
            modeState.metadataChanged = true;
            dashboard.description = s;
          }
        }}
      />
      <div class="flex gap-2">
        <ConfirmButton
          initText="Delete Dashboard"
          confirmText="Confirm Delete"
          onconfirm={removeDashboard}
        />
        {#if modeState.metadataChanged}
          <button
            class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
            onclick={saveDashboard}>Save Title and Description</button
          >
        {/if}
      </div>
      {#if errors.dashboard.save}
        <p class="text-red-500">{errors.dashboard.save}</p>
      {/if}
    </div>
  {/if}
  <div class="grid grid-cols-4 grid-rows-1 gap-2">
    {#if errors.chart.load}
      <p class="text-red-500">{errors.chart.load}</p>
    {:else}
      {#each charts as chart}
        <div
          class={[
            "rounded-lg bg-gray-100 p-3 flex flex-col ",
            col_start[chart.x],
            row_start[chart.y],
            col_span[chart.w],
            row_span[chart.h],
          ]}
        >
          <p class="font-bold text-center">{chart.name}</p>
          {#await asyncTables then tables}
            <p>
              Source Table: {tables.find(
                (t: Table) => t.table_id === chart.table_id,
              )?.name}
            </p>
          {:catch}
            <p>Source Table: <span class="text-red-500">(Not Found)</span></p>
          {/await}
          <ChartComponent {dashboard} {chart} />
          {#if modeState.mode === EditMode.DISPLAY}
            <button
              class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition mt-auto"
              onclick={() => editChart(chart)}>Edit</button
            >
          {:else if modeState.mode === EditMode.EDIT_DASH}
            <ConfirmButton
              class="mt-auto rounded"
              initText="Delete"
              confirmText="Confirm Delete"
              onconfirm={() => {
                removeChart(chart);
              }}
            />
          {/if}
          {#if errors.chart.edit}
            <p class="text-red-500">{errors.chart.edit}</p>
          {/if}
        </div>
      {:else}
        {#if modeState.mode === EditMode.DISPLAY}
          <div class="flex justify-center items-center">
            <p>No Charts.</p>
          </div>
        {/if}
      {/each}
      {#if errors.chart.create}
        <p class="text-red-500">Error: {errors.chart.create}</p>
      {/if}
    {/if}
    {#if modeState.mode === EditMode.EDIT_DASH}
      {#if modeState.newChart}
        <div
          class={[
            "rounded-lg bg-gray-100 flex flex-col gap-3 p-3 ",
            col_start[modeState.newChart.x],
            row_start[modeState.newChart.y],
            col_span[modeState.newChart.w],
            row_span[modeState.newChart.h],
          ]}
        >
          <input bind:value={modeState.newChart.name} />
          <select bind:value={modeState.newChart.chart_kind}>
            {#each Object.values(ChartKind) as kind}
              <option>{kind}</option>
            {/each}
          </select>
          <select bind:value={modeState.newChart.table_id}>
            {#await asyncTables}
              <option value={undefined}>Loading...</option>
            {:then tables}
              {#each tables.filter((t) => t.parent_id == null) as t}
                <option value={t.table_id}>{t.name}</option>
              {/each}
            {/await}
          </select>
          <div class="flex gap-3">
            <button
              onclick={createChart}
              class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
              >Create</button
            >
            <button
              onclick={cancelCreateChart}
              class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition"
              >Cancel</button
            >
          </div>
        </div>
      {/if}
      {#each freeSpaces as space}
        <button
          class={[
            "rounded-lg border border-black border-dashed col-start-{space[0]} row-start-{space[1]} text-center text-3xl font-lg ",
            col_start[space[0]],
            row_start[space[1]],
          ]}
          onclick={() => {
            if (modeState.mode === EditMode.EDIT_DASH) {
              modeState.newChart = blankChart(space[0], space[1], 1, 1);
            }
          }}>+</button
        >
      {/each}
    {/if}
  </div>
  {#if modeState.mode === EditMode.DISPLAY}
    <div class="flex justify-center my-2">
      <button
        class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
        onclick={() => {
          modeEditDash();
        }}>Edit</button
      >
    </div>
  {:else if modeState.mode === EditMode.EDIT_DASH}
    <div class="flex justify-center my-2">
      <button
        class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
        onclick={() => {
          modeDisplay();
        }}>Back</button
      >
    </div>
  {/if}
{:else if modeState.mode === EditMode.EDIT_CHART}
  <input class="mb-2" bind:value={charts[modeState.chartIdx].name} />
  <p class="text-red-500">{errors.chart.save}</p>
  <div class="flex gap-3">
    {#each modeState.axisFields as axis, i}
      <div class="rounded-lg bg-gray-100 p-4 mb-2">
        <div class="flex mb-2 gap-2">
          <p>Field:</p>
          <select bind:value={modeState.axisFields[i].axis.field_id}>
            {#if modeState.chartTableData}
              {#each modeState.chartTableData.fields as field}
                <option value={field.field_id}>{field.name}</option>
              {/each}
            {/if}
          </select>
        </div>
        {#if charts[modeState.chartIdx].chart_kind !== ChartKind.Table}
          <div class="flex gap-2">
            <p>Kind:</p>
            <select bind:value={modeState.axisFields[i].axis.axis_kind}>
              {#each Object.values(AxisKind).filter( (ak) => (modeState.mode === EditMode.EDIT_CHART ? !modeState.axisFields.some((af: AxisField) => af.axis.axis_kind === ak && axis.axis.axis_id !== af.axis.axis_id) : true), ) as kind}
                <option>{kind}</option>
              {/each}
            </select>
          </div>
        {/if}
        <div class="flex gap-2">
          <p>Aggregate:</p>
          <select bind:value={modeState.axisFields[i].axis.aggregate}>
            <option value={null}>None</option>
            {#each Object.values(Aggregate) as agg}
              <option>{agg}</option>
            {/each}
          </select>
        </div>
        <ConfirmButton
          initText="Delete"
          confirmText="Confirm Delete"
          onconfirm={() => {
            if (modeState.mode === EditMode.EDIT_CHART) {
              modeState.axisFields.splice(i, 1);
            }
          }}
        />
        {#if errors.axes.save[axis.axis.axis_id.toString()]}<p
            class="text-red-500"
          >
            {errors.axes.save[axis.axis.axis_id.toString()]}
          </p>{/if}
      </div>
    {/each}
  </div>
  <div class="flex gap-2">
    {#if modeState.axisFields.length < Object.values(AxisKind).length || charts[modeState.chartIdx].chart_kind === ChartKind.Table}
      <button
        class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
        onclick={() => {
          if (modeState.mode === EditMode.EDIT_CHART) {
            modeState.axisFields.push({
              axis: blankAxis(charts[modeState.chartIdx]),
              field_name: "",
              field_kind: null as unknown as FieldKind,
            });
          }
        }}>Add Axis</button
      >
    {/if}
    <button
      class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
      onclick={() => {
        if (modeState.mode === EditMode.EDIT_CHART) {
          saveChartWithAxisFields(
            charts[modeState.chartIdx],
            modeState.axisFields,
          );
        }
      }}>Save</button
    >
    <button
      onclick={cancelEditChart}
      class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition"
      >Cancel</button
    >
  </div>
{/if}

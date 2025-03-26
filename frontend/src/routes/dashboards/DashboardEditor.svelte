<script lang="ts">
  import {
    deleteChart,
    getChartData,
    getCharts,
    getTableData,
    getTables,
    patchChart,
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

  const EditMode = {
    DISPLAY: 0,
    DASH: 1,
    CHART: 2,
  };

  //
  // State
  //

  let editMode = $state(EditMode.DISPLAY);

  let charts: Chart[] = $state([]);
  let loadChartError = $state("");

  let freeSpaces = $derived.by(() => {
    let out = [];
    for (let i = 1; i <= 4; i++) {
      for (let j = 1; j <= 1; j++) {
        if (
          !charts.some((c) => withinChart(i, j, c)) &&
          (!newChart || !withinChart(i, j, newChart))
        ) {
          out.push([i, j]);
        }
      }
    }

    return out;
  });

  let newChart: Chart | null = $state(null);
  let createChartError = $state("");

  // chart being edited
  let curChartIdx = $state(-1);
  let editedAxisFields = $state([] as AxisField[]);
  let curChartTableData: TableData | null = $state(null);
  let editChartError = $state("");

  let saveChartError = $state("");
  let saveAxesError = $state("");
  $inspect(editedAxisFields);

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
    while (
      editedAxisFields.some((af) => af.axis.axis_kind === kinds[i]) &&
      i < kinds.length
    ) {
      i++;
    }

    let j = -1;
    while (editedAxisFields.some((af) => af.axis.axis_id === j)) {
      j--;
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
    newChart = null;
  };

  const cancelEditChart = () => {
    editMode = EditMode.DISPLAY;
    curChartIdx = -1;
    editedAxisFields = [];
    curChartTableData = null;
  };

  //
  // API
  //

  const loadCharts = () =>
    getCharts(dashboard)
      .then((result: Chart[]) => {
        loadChartError = "";
        charts = result.map((c, i) => {
          c.x = i + 1;
          c.y = 1;
          c.w = 1;
          c.h = 1;
          return c;
        });
      })
      .catch((e) => {
        loadChartError = e.body.toString();
      });

  const createChart = () => {
    if (newChart) {
      postChart(dashboard, newChart)
        .then(loadCharts)
        .then(cancelCreateChart)
        .then(() => {
          createChartError = "";
          editMode = EditMode.DISPLAY;
        })
        .catch((e) => {
          createChartError = e.body.toString();
        });
    }
  };

  const editChart = (c: Chart) => {
    getChartData(dashboard, c)
      .then(async (r) => {
        editMode = EditMode.CHART;
        curChartIdx = charts.findIndex((d) => d.chart_id === c.chart_id);
        editedAxisFields = r.axes;
        curChartTableData = await getTables().then((t) =>
          getTableData(
            t.find((table) => table.table_id === r.chart.table_id) as Table,
          ).catch(() => null),
        );
        if (curChartTableData === null)
          throw { body: "Could not get Chart data" };

        editChartError = "";
        saveAxesError = "";
      })
      .catch((e) => {
        editChartError = e.body.toString();
      });
  };

  const removeChart = (c: Chart) => {
    deleteChart(dashboard, c)
      .then(() => {
        loadCharts();
        editChartError = "";
      })
      .catch((e) => {
        editChartError = e.body.toString();
      });
  };

  const saveChartWithAxisFields = (chart: Chart, axes: AxisField[]) => {
    let chartPromise = patchChart(dashboard, chart)
      .then(() => {
        saveChartError = "";
      })
      .catch((e) => {
        saveChartError = e.body.toString();
        throw Error();
      });
    let axisPromise = putAxes(
      dashboard,
      chart,
      axes.map((af) => af.axis),
    )
      .then(() => {
        saveAxesError = "";
      })
      .catch((e) => {
        saveAxesError = e.body.toString();
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

{#if editMode === EditMode.DISPLAY || editMode === EditMode.DASH}
  {#if editMode === EditMode.DISPLAY}
    <div class="flex flex-col items-center">
      <h2 class="font-bold text-xl">{dashboard.name}</h2>
      <p>{dashboard.description}</p>
    </div>
  {:else if editMode === EditMode.DASH}
    <div class="flex flex-col items-center">
      <input bind:value={dashboard.name} />
      <input bind:value={dashboard.description} />
    </div>
  {/if}
  <div class="grid grid-cols-4 grid-rows-1 gap-2">
    {#if loadChartError}
      <p class="text-red-500">{loadChartError}</p>
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
          {#if editMode === EditMode.DISPLAY}
            <button
              class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition mt-auto"
              onclick={() => editChart(chart)}>Edit</button
            >
          {:else if editMode === EditMode.DASH}
            <ConfirmButton
              class="mt-auto rounded"
              initText="Delete"
              confirmText="Confirm Delete"
              onconfirm={() => {
                removeChart(chart);
              }}
            />
          {/if}
          {#if editChartError}
            <p class="text-red-500">{editChartError}</p>
          {/if}
        </div>
      {:else}
        {#if editMode === EditMode.DISPLAY}
          <div class="flex justify-center items-center">
            <p>No Charts.</p>
          </div>
        {/if}
      {/each}
      {#if createChartError}
        <p class="text-red-500">Error: {createChartError}</p>
      {/if}
    {/if}
    {#if editMode === EditMode.DASH}
      {#if newChart}
        <div
          class={[
            "rounded-lg bg-gray-100 flex flex-col gap-3 p-3 ",
            col_start[newChart.x],
            row_start[newChart.y],
            col_span[newChart.w],
            row_span[newChart.h],
          ]}
        >
          <input bind:value={newChart.name} />
          <select bind:value={newChart.chart_kind}>
            {#each Object.values(ChartKind) as kind}
              <option>{kind}</option>
            {/each}
          </select>
          <select bind:value={newChart.table_id}>
            {#await asyncTables}
              <option value={undefined}>Loading...</option>
            {:then tables}
              {#each tables as t}
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
            newChart = blankChart(space[0], space[1], 1, 1);
          }}>+</button
        >
      {/each}
    {/if}
  </div>
  {#if editMode === EditMode.DISPLAY}
    <div class="flex justify-center my-2">
      <button
        class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
        onclick={() => {
          editMode = EditMode.DASH;
        }}>Edit</button
      >
    </div>
  {:else if editMode === EditMode.DASH}
    <div class="flex justify-center my-2">
      <button
        class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
        onclick={() => {
          editMode = EditMode.DISPLAY;
        }}>Back</button
      >
      <ConfirmButton
        initText="Delete Dashboard"
        confirmText="Confirm Delete"
        onconfirm={removeDashboard}
      />
    </div>
  {/if}
{:else}
  <input class="mb-2" bind:value={charts[curChartIdx].name} />
  <p class="text-red-500">{saveChartError}</p>
  <div class="flex gap-3">
    {#each editedAxisFields as axis, i}
      <div class="rounded-lg bg-gray-100 p-4 mb-2">
        <div class="flex mb-2 gap-2">
          <p>Field:</p>
          <select bind:value={editedAxisFields[i].axis.field_id}>
            {#if curChartTableData}
              {#each curChartTableData.fields as field}
                <option value={field.field_id}>{field.name}</option>
              {/each}
            {/if}
          </select>
        </div>
        {#if charts[curChartIdx].chart_kind !== ChartKind.Table}
          <div class="flex gap-2">
            <p>Kind:</p>
            <select bind:value={editedAxisFields[i].axis.axis_kind}>
              {#each Object.values(AxisKind).filter((ak) => !editedAxisFields.some((af) => af.axis.axis_kind === ak && axis.axis.axis_id !== af.axis.axis_id)) as kind}
                <option>{kind}</option>
              {/each}
            </select>
          </div>
        {/if}
        <div class="flex gap-2">
          <p>Aggregate:</p>
          <select bind:value={editedAxisFields[i].axis.aggregate}>
            <option value={null}>None</option>
            {#each Object.values(Aggregate) as agg}
              <option>{agg}</option>
            {/each}
          </select>
        </div>
        <ConfirmButton
          initText="Delete"
          confirmText="Confirm Delete"
          onconfirm={() => editedAxisFields.splice(i, 1)}
        />
      </div>
    {/each}
  </div>
  <div class="flex gap-2">
    {#if editedAxisFields.length < Object.values(AxisKind).length || charts[curChartIdx].chart_kind === ChartKind.Table}
      <button
        class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
        onclick={() => {
          editedAxisFields.push({
            axis: blankAxis(charts[curChartIdx]),
            field_name: "",
            field_kind: null as unknown as FieldKind,
          });
        }}>Add Axis</button
      >
    {/if}
    <button
      class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
      onclick={() => {
        saveChartWithAxisFields(charts[curChartIdx], editedAxisFields);
      }}>Save</button
    >
    <button
      onclick={cancelEditChart}
      class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition"
      >Cancel</button
    >
  </div>
  {#if saveAxesError}<p class="text-red-500">{saveAxesError}</p>{/if}
{/if}

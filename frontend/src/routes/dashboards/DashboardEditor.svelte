<script lang="ts">
  import {
    getChartData,
    getCharts,
    getDataTable,
    getTables,
    postChart,
    putAxes,
  } from "$lib/api";
  import {
    type Dashboard,
    type Chart,
    type Axis,
    ChartKind,
    AxisKind,
    type Table,
    type DataTable,
    type AxisField,
    type FieldKind,
  } from "$lib/types.d.js";
  import { onMount } from "svelte";

  let { dashboard }: { dashboard: Dashboard } = $props();

  //
  // Constants
  //

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

  let gridDims = $derived({
    w: charts.length + (editMode === EditMode.DASH ? 1 : 0) || 1,
    h: 1,
  });

  let freeSpaces = $derived.by(() => {
    let out = [];
    for (let i = 1; i <= gridDims.w; i++) {
      for (let j = 1; j <= gridDims.h; j++) {
        if (
          !charts.some(
            (c) =>
              withinChart(i, j, c) &&
              (!newChart || !withinChart(i, j, newChart)),
          )
        ) {
          out.push([i, j]);
        }
      }
    }

    return out;
  });

  let newChart: Chart | null = $state(null);

  // chart being edited
  let curChartIdx = $state(-1);
  let editedAxisFields = $state([] as AxisField[]);
  let curChartTableData: DataTable | null = $state(null);
  $inspect(editedAxisFields);
  //
  // Helper methods
  //
  const blankChart = (x: number, y: number, w: number, h: number): Chart => {
    let i = 0;
    while (charts.some((c) => c.title === `Chart ${++i}`));
    return {
      chart_id: -1,
      dashboard_id: dashboard.dashboard_id,
      table_id: -1,
      title: `Chart ${i}`,
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
      axis_kind: kinds[i],
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
    getCharts(dashboard).then((result: Chart[]) => {
      charts = result.map((c, i) => {
        c.x = i + 1;
        c.y = 1;
        c.w = 1;
        c.h = 1;
        return c;
      });
    });

  const createChart = () => {
    if (newChart) {
      postChart(dashboard, newChart)
        .then(loadCharts)
        .then(cancelCreateChart)
        .then(() => {
          editMode = EditMode.DISPLAY;
        });
    }
  };

  const editChart = (c: Chart) => {
    getChartData(dashboard, c).then(async (r) => {
      editMode = EditMode.CHART;
      curChartIdx = charts.findIndex((d) => d.chart_id === c.chart_id);
      editedAxisFields = r.axes;
      curChartTableData = await getTables().then((t) =>
        getDataTable(
          t.find((table) => table.table_id === r.chart.table_id) as Table,
        ),
      );
    });
  };

  const saveAxisFields = (chart: Chart, axes: AxisField[]) =>
    putAxes(
      dashboard,
      chart,
      axes.map((af) => af.axis),
    ).then(cancelEditChart);

  //
  // Startup
  //

  onMount(() => {
    loadCharts();
  });
</script>

{#if editMode === EditMode.DISPLAY || editMode === EditMode.DASH}
  <div class="grid grid-cols-{gridDims.w} grid-rows-{gridDims.h} gap-2">
    {#each charts as chart}
      <div
        class="rounded-lg bg-gray-100 p-3 col-start-{chart.x} col-span-{chart.w} row-start-{chart.y} row-span-{chart.h} flex flex-col"
      >
        <p class="font-bold text-center">{chart.title}</p>
        <p>Kind: {chart.chart_kind}</p>
        {#await asyncTables then tables}
          <p>
            Source Table: {tables.find(
              (t: Table) => t.table_id === chart.table_id,
            )?.name}
          </p>
        {/await}
        <button
          class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
          onclick={() => editChart(chart)}>Edit</button
        >
      </div>
    {:else}
      {#if editMode === EditMode.DISPLAY}
        <div class="flex justify-center items-center">
          <p>No Charts.</p>
        </div>
      {/if}
    {/each}
    {#if editMode === EditMode.DASH}
      {#if newChart}
        <div
          class="rounded-lg bg-gray-100 col-start-{newChart.x} col-span-{newChart.w} row-start-{newChart.y} row-span-{newChart.h} flex flex-col gap-3"
        >
          <input bind:value={newChart.title} />
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
          class="rounded-lg border border-black border-dashed col-start-{space[0]} row-start-{space[1]} text-center text-3xl font-lg"
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
  {/if}
{:else}
  <input class="mb-2" bind:value={charts[curChartIdx].title} />
  <div class="flex gap-3">
    {#each editedAxisFields as axis, i}
      <div class="rounded-lg bg-gray-100 p-4 mb-2">
        <div class="flex mb-2">
          <p>Field:</p>
          <select bind:value={editedAxisFields[i].axis.field_id}>
            {#if curChartTableData}
              {#each curChartTableData.fields as field}
                <option value={field.field_id}>{field.name}</option>
              {/each}
            {/if}
          </select>
        </div>
        <div class="flex">
          <p>Kind:</p>
          <select bind:value={editedAxisFields[i].axis.axis_kind}>
            {#each Object.values(AxisKind).filter((ak) => !editedAxisFields.some((af) => af.axis.axis_kind === ak && axis.axis.axis_id !== af.axis.axis_id)) as kind}
              <option>{kind}</option>
            {/each}
          </select>
        </div>
      </div>
    {/each}
  </div>
  <div class="flex gap-2">
    {#if editedAxisFields.length < Object.values(AxisKind).length}
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
        saveAxisFields(charts[curChartIdx], editedAxisFields);
      }}>Save</button
    >
  </div>
{/if}

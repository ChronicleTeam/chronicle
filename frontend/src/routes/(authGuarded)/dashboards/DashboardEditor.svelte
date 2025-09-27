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
  import ChartComponent from "$lib/components/charts/ChartDisplay.svelte";
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
  } from "$lib/types";
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

  // list of fetched tables
  const asyncTables = $state(getTables());

  //
  // State
  //

  // mode-dependent variables
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

  // list of charts associated with dashboard
  let charts: Chart[] = $state([]);

  // error fields
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

  /**
   * Get a new blank chart
   * @returns a new Chart
   */
  const blankChart = (): Chart => {
    let i = 0;
    while (charts.some((c) => c.name === `Chart ${++i}`));
    return {
      chart_id: -1,
      dashboard_id: dashboard.dashboard_id,
      table_id: -1,
      name: `Chart ${i}`,
      chart_kind: ChartKind.Bar,
    };
  };

  /**
   * Get a new blank axis
   * @returns a new Axis
   */
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

  /**
   * Cancel chart creation
   */
  const cancelCreateChart = () => {
    if (modeState.mode === EditMode.EDIT_DASH) {
      modeState.newChart = null;
    }
  };

  /**
   * Cancel chart editing
   */
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
        charts = result;
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
  <!-- Dashboard viewer / editor -->

  <!-- Dashboard metadata -->
  {#if modeState.mode === EditMode.DISPLAY}
    <div class="flex flex-col items-center">
      <h2 class="font-bold text-xl">{dashboard.name}</h2>
      <p>{dashboard.description}</p>
    </div>
  {:else if modeState.mode === EditMode.EDIT_DASH}
    <div class="flex flex-col gap-1 items-center">
      <label class="input">
        Name:
        <input
          bind:value={
            () => dashboard.name,
            (s) => {
              if (modeState.mode === EditMode.EDIT_DASH) {
                modeState.metadataChanged = true;
                dashboard.name = s;
              }
            }
          }
        />
      </label>
      <label class="input">
        Description:
        <input
          bind:value={
            () => dashboard.description,
            (s) => {
              if (modeState.mode === EditMode.EDIT_DASH) {
                modeState.metadataChanged = true;
                dashboard.description = s;
              }
            }
          }
        />
      </label>
      <div class="flex gap-2">
        <ConfirmButton
          initText="Delete Dashboard"
          confirmText="Confirm Delete"
          onconfirm={removeDashboard}
        />
        {#if modeState.metadataChanged}
          <button class="btn" onclick={saveDashboard}
            >Save Title and Description</button
          >
        {/if}
      </div>
      {#if errors.dashboard.save}
        <p class="text-error">{errors.dashboard.save}</p>
      {/if}
    </div>
  {/if}

  <!-- Chart grid/list -->
  <div class="grid grid-cols-4 grid-rows-1 gap-2 mt-2 h-80">
    {#if errors.chart.load}
      <p class="text-error">{errors.chart.load}</p>
    {:else}
      {#each charts as chart}
        <div
          class="card bg-base-100 p-3 flex flex-col justify-between shadow-sm"
        >
          <!-- Chart info -->
          <div>
            <p class="font-bold text-center">{chart.name}</p>
            {#await asyncTables then tables}
              <p>
                Source Table: {tables.find(
                  (t: Table) => t.table_id === chart.table_id,
                )?.name}
              </p>
            {:catch}
              <p>Source Table: <span class="text-error">(Not Found)</span></p>
            {/await}
          </div>

          <!-- Chart -->
          <ChartComponent {dashboard} {chart} />

          <!-- Buttons -->
          <div>
            {#if modeState.mode === EditMode.DISPLAY}
              <button class="btn btn-block" onclick={() => editChart(chart)}
                >Edit</button
              >
            {:else if modeState.mode === EditMode.EDIT_DASH}
              <ConfirmButton
                class="btn btn-block"
                initText="Delete"
                confirmText="Confirm Delete"
                onconfirm={() => {
                  removeChart(chart);
                }}
              />
            {/if}
            {#if errors.chart.edit}
              <p class="text-error">{errors.chart.edit}</p>
            {/if}
          </div>
        </div>
      {:else}
        {#if modeState.mode === EditMode.DISPLAY}
          <div class="flex justify-center items-center">
            <p>No Charts.</p>
          </div>
        {/if}
      {/each}
      {#if errors.chart.create}
        <p class="text-error">Error: {errors.chart.create}</p>
      {/if}
    {/if}
    {#if modeState.mode === EditMode.EDIT_DASH}
      <!-- Chart creation input -->
      {#if modeState.newChart}
        <div
          class="card bg-base-100 p-3 flex flex-col gap-3 justify-between shadow-sm"
        >
          <!-- Name -->
          <input class="input w-full" bind:value={modeState.newChart.name} />

          <!-- Chart kind -->
          <div class="flex gap-2 justify-between items-center">
            <label for="new-chart-kind-sel">Kind: </label>
            <select
              id="new-chart-kind-sel"
              class="select"
              bind:value={modeState.newChart.chart_kind}
            >
              {#each Object.values(ChartKind) as kind}
                <option>{kind}</option>
              {/each}
            </select>
          </div>

          <!-- Source Table -->
          <div class="flex gap-2 justify-between items-center">
            <label for="new-chart-table-sel">Table: </label>
            <select
              id="new-chart-table-sel"
              class="select"
              bind:value={modeState.newChart.table_id}
            >
              {#await asyncTables}
                <option value={undefined}>Loading...</option>
              {:then tables}
                {#each tables.filter((t) => t.parent_id == null) as t}
                  <option value={t.table_id}>{t.name}</option>
                {/each}
              {/await}
            </select>
          </div>

          <!-- Buttons -->
          <div class="flex gap-3 justify-center mt-auto">
            <button onclick={createChart} class="btn">Create</button>
            <button onclick={cancelCreateChart} class="btn btn-error"
              >Cancel</button
            >
          </div>
        </div>

        <!-- "Add Chart" button -->
      {:else}
        <button
          class={"btn btn-dash border-2 col-start-{space[0]} row-start-{space[1]} text-center text-3xl h-full font-lg"}
          onclick={() => {
            if (modeState.mode === EditMode.EDIT_DASH) {
              modeState.newChart = blankChart();
            }
          }}>+</button
        >
      {/if}
    {/if}
  </div>

  <!-- Edit dashboard / back button -->
  {#if modeState.mode === EditMode.DISPLAY}
    <div class="flex justify-center my-2">
      <button
        class="btn"
        onclick={() => {
          modeEditDash();
        }}>Edit</button
      >
    </div>
  {:else if modeState.mode === EditMode.EDIT_DASH}
    <div class="flex justify-center my-2">
      <button
        class="btn"
        onclick={() => {
          modeDisplay();
        }}>Back</button
      >
    </div>
  {/if}
{:else if modeState.mode === EditMode.EDIT_CHART}
  <!-- Chart editor (Axes) -->

  <!-- Chart metadata -->
  <input class="input mb-2" bind:value={charts[modeState.chartIdx].name} />
  <p class="text-error">{errors.chart.save}</p>

  <!-- Axes -->
  <div class="flex gap-3">
    {#each modeState.axisFields as axis, i}
      <div class="card bg-base-100 shadow-sm p-4 mb-2">
        <!-- Field -->
        <div class="flex mb-2 gap-2 items-center justify-between">
          <p>Field:</p>
          <select
            class="select"
            bind:value={modeState.axisFields[i].axis.field_id}
          >
            {#if modeState.chartTableData}
              {#each modeState.chartTableData.fields as field}
                <option value={field.field_id}>{field.name}</option>
              {/each}
            {/if}
          </select>
        </div>

        <!-- Axis Kind -->
        {#if charts[modeState.chartIdx].chart_kind !== ChartKind.Table}
          <div class="flex mb-2 gap-2 justify-between">
            <p>Kind:</p>
            <select
              class="select items-center"
              bind:value={modeState.axisFields[i].axis.axis_kind}
            >
              {#each Object.values(AxisKind).filter( (ak) => (modeState.mode === EditMode.EDIT_CHART ? !modeState.axisFields.some((af: AxisField) => af.axis.axis_kind === ak && axis.axis.axis_id !== af.axis.axis_id) : true), ) as kind}
                <option>{kind}</option>
              {/each}
            </select>
          </div>
        {/if}

        <!-- Aggregation type -->
        <div class="flex mb-2 gap-2 justify-between">
          <p>Aggregate:</p>
          <select
            class="select items-center"
            bind:value={modeState.axisFields[i].axis.aggregate}
          >
            <option value={null}>None</option>
            {#each Object.values(Aggregate) as agg}
              <option>{agg}</option>
            {/each}
          </select>
        </div>

        <!-- Button -->
        <div class="flex gap-2 justify-center">
          <ConfirmButton
            initText="Delete"
            confirmText="Confirm Delete"
            onconfirm={() => {
              if (modeState.mode === EditMode.EDIT_CHART) {
                modeState.axisFields.splice(i, 1);
              }
            }}
          />
        </div>
        {#if errors.axes.save[axis.axis.axis_id.toString()]}<p
            class="text-error"
          >
            {errors.axes.save[axis.axis.axis_id.toString()]}
          </p>{/if}
      </div>
    {/each}
  </div>

  <!-- Button cluster -->
  <div class="flex gap-2">
    {#if modeState.axisFields.length < Object.values(AxisKind).length || charts[modeState.chartIdx].chart_kind === ChartKind.Table}
      <button
        class="btn"
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
      class="btn"
      onclick={() => {
        if (modeState.mode === EditMode.EDIT_CHART) {
          saveChartWithAxisFields(
            charts[modeState.chartIdx],
            modeState.axisFields,
          );
        }
      }}>Save</button
    >
    <button onclick={cancelEditChart} class="btn btn-error">Cancel</button>
  </div>
{/if}

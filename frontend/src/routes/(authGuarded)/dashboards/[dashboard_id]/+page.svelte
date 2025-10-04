<script lang="ts">
  import {
    deleteChart,
    getCharts,
    patchDashboard,
    postChart,
    deleteDashboard,
  } from "$lib/api";
  import ChartComponent from "$lib/components/charts/ChartDisplay.svelte";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import {
    type Dashboard,
    type Chart,
    ChartKind,
    type Table,
  } from "$lib/types";
  import type { ModeState } from "./types";
  import { EditMode } from "./types";
  import { goto, invalidateAll } from "$app/navigation";

  let { data } = $props();

  //
  // Constants
  //

  // list of fetched tables
  let tables = $derived(data.tables);
  let dashboard: Dashboard = $derived(data.dashboard);
  let charts: Chart[] = $derived(data.charts);

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

  // list of charts associated with dashboard

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
   * Cancel chart creation
   */
  const cancelCreateChart = () => {
    if (modeState.mode === EditMode.EDIT_DASH) {
      modeState.newChart = null;
    }
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
    getCharts(dashboard.dashboard_id.toString())
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
        .then(cancelCreateChart)
        .then(() => {
          errors.chart.create = "";
          modeDisplay();
          invalidateAll();
        })
        .catch((e) => {
          errors.chart.create = e.body.toString();
        });
    }
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

  const removeDashboard = () => {
    deleteDashboard(dashboard)
      .then(async () => {
        await goto("/dashboards");
      })
      .then(invalidateAll)
      .catch((e) => {
        errors.dashboard.save = e.body.toString();
      });
  };
</script>

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
      <div class="card bg-base-100 p-3 flex flex-col justify-between shadow-sm">
        <!-- Chart info -->
        <div>
          <p class="font-bold text-center">{chart.name}</p>
          <p>
            Source Table: {tables.find(
              (t: Table) => t.table_id === chart.table_id,
            )?.name}
          </p>
        </div>

        <!-- Chart -->
        <ChartComponent {dashboard} {chart} />

        <!-- Buttons -->
        <div>
          {#if modeState.mode === EditMode.DISPLAY}
            <button
              class="btn btn-block"
              onclick={() =>
                goto(
                  `/dashboards/${dashboard.dashboard_id}/charts/${chart.chart_id}/edit`,
                )}>Edit</button
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
            {#each tables.filter((t) => t.parent_id == null) as t}
              <option value={t.table_id}>{t.name}</option>
            {/each}
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
        invalidateAll();
        modeDisplay();
      }}>Back</button
    >
  </div>
{/if}

<script lang="ts">
  import { patchChart, putAxes } from "$lib/api";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import {
    type Dashboard,
    type Chart,
    type Axis,
    ChartKind,
    AxisKind,
    type TableData,
    type AxisField,
    type FieldKind,
    type ChartData,
    Aggregate,
  } from "$lib/types";
  import { goto } from "$app/navigation";

  let { data } = $props();

  //
  // Constants
  //

  let dashboard: Dashboard = $derived(data.dashboard);
  let chartData: ChartData = $derived(data.chartData);
  let chart: Chart = $derived(chartData.chart);
  let tableData: TableData = $derived(data.tableData);

  //
  // State
  //

  let axisFields: AxisField[] = $state(chartData.axes);
  $effect(() => {
    axisFields = chartData.axes;
  });

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
   * Get a new blank axis
   * @returns a new Axis
   */
  const blankAxis = (c: Chart): Axis => {
    let kinds = Object.values(AxisKind);
    let i = 0;
    while (
      axisFields.some((af) => af.axis.axis_kind === kinds[i]) &&
      i < kinds.length
    ) {
      i++;
    }

    let j = -1;
    while (axisFields.some((af) => af.axis.axis_id === j)) {
      j--;
    }

    return {
      axis_id: j,
      chart_id: c.chart_id,
      field_id: -1,
      axis_kind: c.chart_kind === ChartKind.Table ? AxisKind.Label : kinds[i],
    };
  };

  /**
   * Cancel chart editing
   */
  const cancelEditChart = () => {
    goto(`/dashboards/${dashboard.dashboard_id}`);
  };

  //
  // API
  //

  const saveChartAndAxisFields = async () => {
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
      axisFields.map((af) => af.axis),
    )
      .then(() => {
        errors.axes.save = {};
      })
      .catch((e) => {
        errors.axes.save = e.body;
        throw Error();
      });

    await Promise.all([chartPromise, axisPromise]).then(() => {
      cancelEditChart();
    });
  };
</script>

<!-- Chart metadata -->
<input class="input mb-2" bind:value={chart.name} />
<p class="text-error">{errors.chart.save}</p>

<!-- Axes -->
<div class="flex gap-3">
  {#each axisFields as axis, i}
    <div class="card bg-base-100 shadow-sm p-4 mb-2">
      <!-- Field -->
      <div class="flex mb-2 gap-2 items-center justify-between">
        <p>Field:</p>
        <select class="select" bind:value={axisFields[i].axis.field_id}>
          {#each tableData.fields as field}
            <option value={field.field_id}>{field.name}</option>
          {/each}
        </select>
      </div>

      <!-- Axis Kind -->
      {#if chart.chart_kind !== ChartKind.Table}
        <div class="flex mb-2 gap-2 justify-between">
          <p>Kind:</p>
          <select
            class="select items-center"
            bind:value={axisFields[i].axis.axis_kind}
          >
            {#each Object.values(AxisKind).filter((ak) => !axisFields.some((af: AxisField) => af.axis.axis_kind === ak && axis.axis.axis_id !== af.axis.axis_id)) as kind}
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
          bind:value={axisFields[i].axis.aggregate}
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
            axisFields.splice(i, 1);
          }}
        />
      </div>
      {#if errors.axes.save[axis.axis.axis_id.toString()]}<p class="text-error">
          {errors.axes.save[axis.axis.axis_id.toString()]}
        </p>{/if}
    </div>
  {/each}
</div>

<!-- Button cluster -->
<div class="flex gap-2">
  {#if axisFields.length < Object.values(AxisKind).length || chart.chart_kind === ChartKind.Table}
    <button
      class="btn"
      onclick={() => {
        axisFields.push({
          axis: blankAxis(chart),
          field_name: "",
          field_kind: null as unknown as FieldKind,
        });
      }}>Add Axis</button
    >
  {/if}
  <button class="btn" onclick={saveChartAndAxisFields}>Save</button>
  <button class="btn btn-error" onclick={cancelEditChart}>Cancel</button>
</div>

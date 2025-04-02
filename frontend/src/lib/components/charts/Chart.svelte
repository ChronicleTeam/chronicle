<script lang="ts">
  import { getChartData } from "$lib/api";
  import {
    AxisKind,
    ChartKind,
    type Chart,
    type ChartData,
    type Dashboard,
    type Cells,
    type AxisField,
    FieldType,
  } from "$lib/types";
  import { Chart as ChartGraphic } from "chart.js/auto";
  import { onMount } from "svelte";
  let { dashboard, chart }: { dashboard: Dashboard; chart: Chart } = $props();

  // fields that the Line chart should sort
  const SORTABLE_FIELDS = [
    FieldType.Integer,
    FieldType.Decimal,
    FieldType.Progress,
    FieldType.DateTime,
  ];

  // data to use for the chart
  let chartData: ChartData | null = $state(null);

  // error state
  let error = $state("");

  // determines whether modal is active or not
  let isModalActive = $state(false);

  let g: any;

  // Update graph on change
  $effect(() => {
    if (chartData) {
      if (chartData.chart.chart_kind === ChartKind.Table) {
      } else {
        // Identify axes
        let xAxis = chartData.axes.find((a) => a.axis.axis_kind === AxisKind.X);
        let yAxis = chartData.axes.find((a) => a.axis.axis_kind === AxisKind.Y);
        let colorAxis = chartData.axes.find(
          (a) => a.axis.axis_kind === AxisKind.Color,
        );
        let sizeAxis = chartData.axes.find(
          (a) => a.axis.axis_kind === AxisKind.Size,
        );
        let tooltipAxis = chartData.axes.find(
          (a) => a.axis.axis_kind === AxisKind.Tooltip,
        );
        let labelAxis = chartData.axes.find(
          (a) => a.axis.axis_kind === AxisKind.Label,
        );

        // ensure x and y axes exist
        if (!xAxis || !yAxis) return;

        let options = {
          scales: {
            x: {
              title: {
                display: true,
                text: xAxis.field_name,
              },
            },
            y: {
              title: {
                display: true,
                text: yAxis.field_name,
              },
            },
          },
          plugins: {
            legend: {
              display: false,
            },
          },
        };

        switch (chartData.chart.chart_kind) {
          case ChartKind.Bar:
            new ChartGraphic(g, {
              type: "bar",
              data: {
                labels: chartData.cells.map(
                  (row: Cells) =>
                    row[xAxis.axis.axis_id ?? -1]?.toString() ?? "",
                ),
                datasets: [
                  {
                    label: "data",
                    data: chartData.cells.map(
                      (row: Cells) => row[yAxis.axis.axis_id],
                    ),
                  },
                ],
              },
              options,
            });
            break;
          case ChartKind.Line:
            // sort data first
            let sortedCells = chartData.cells;
            if (SORTABLE_FIELDS.some((t) => xAxis.field_kind.type === t)) {
              sortedCells = chartData.cells.toSorted(
                (rowA: Cells, rowB: Cells) => {
                  return (rowA[xAxis.axis.axis_id] ?? 1) >
                    (rowB[xAxis.axis.axis_id] ?? 1)
                    ? 1
                    : -1;
                },
              );
            }

            // generate graph
            new ChartGraphic(g, {
              type: "line",
              data: {
                labels: sortedCells.map(
                  (row: Cells) =>
                    row[xAxis.axis.axis_id ?? -1]?.toString() ?? "",
                ),
                datasets: [
                  {
                    label: "data",
                    data: sortedCells.map(
                      (row: Cells) => row[yAxis.axis.axis_id],
                    ),
                    tension: 0.1,
                  },
                ],
              },
              options,
            });
            break;
        }
      }
    }
  });

  //
  // Helper method
  //

  /**
   * Turn Date objects in ChartData to human-readable strings
   * @param {ChartData} c - ChartData object to stringify
   * @returns {ChartData} - ChartData object with human-readable strings
   */
  const stringifyDates = (c: ChartData): ChartData => {
    c.axes.forEach((a: AxisField) => {
      if (a.field_kind.type === FieldType.DateTime) {
        c.cells = c.cells.map((row: Cells) => {
          row[a.axis.axis_id] =
            `${(row[a.axis.axis_id] as Date).getFullYear()}-${(row[a.axis.axis_id] as Date).getMonth() + 1}-${(row[a.axis.axis_id] as Date).getUTCDate()}`;
          return row;
        });
      }
    });

    return c;
  };

  //
  // Table stuff
  //

  // the column used as basis for sorting
  let selectedColumn = $state({ axis_id: -1, ascending: true });

  // the method used to sort table rows
  let sortingMethod = $derived((rowA: Cells, rowB: Cells) =>
    selectedColumn.axis_id in rowA && selectedColumn.axis_id in rowB
      ? selectedColumn.ascending
        ? (rowA[selectedColumn.axis_id] ?? 1) >
          (rowB[selectedColumn.axis_id] ?? 1)
          ? 1
          : -1
        : (rowA[selectedColumn.axis_id] ?? 1) <
            (rowB[selectedColumn.axis_id] ?? 1)
          ? 1
          : -1
      : 1,
  );

  // the cells to display if ChartKind is Table
  let tableCells = $derived.by(() => {
    if (chartData) {
      return chartData.cells.toSorted(sortingMethod);
    }
  });

  //
  // Startup
  //

  onMount(() => {
    getChartData(dashboard, chart)
      .then((r: ChartData) => {
        chartData = r;
        chartData = stringifyDates(chartData);
      })
      .catch((e) => {
        error = e.body.toString();
      });
  });
</script>

<div
  class={isModalActive
    ? "z-10 size-full fixed top-0 left-0 bg-black/25 flex justify-center items-center" // add modal styling when modal is active
    : ""}
  onclick={() => {
    isModalActive = false;
  }}
>
  <div
    onclick={(e) => {
      e.stopPropagation();
      isModalActive = true;
    }}
    class={isModalActive
      ? "bg-white rounded-lg p-3 size-1/2 transition-all flex justify-center"
      : "transition-all flex justify-center"}
  >
    {#if error}
      <p class="text-red-500">({error})</p>
    {:else if chartData && !(chartData.chart.chart_kind === ChartKind.Table)}
      <!-- Bar or Line type Chart -->
      <div class="size-full flex justify-center items-center">
        <canvas bind:this={g}></canvas>
      </div>
    {:else if chartData}
      <!-- Table type Chart -->
      <table class="border border-black">
        <thead>
          <tr>
            {#each chartData.axes as axis}
              <th
                class="border border-black bg-white select-none"
                onclick={(e) => {
                  e.stopPropagation();
                  if (selectedColumn.axis_id === axis.axis.axis_id) {
                    selectedColumn.ascending = !selectedColumn.ascending;
                  } else {
                    selectedColumn.axis_id = axis.axis.axis_id;
                    selectedColumn.ascending = true;
                  }
                }}
                >{axis.field_name}{selectedColumn.axis_id === axis.axis.axis_id
                  ? selectedColumn.ascending
                    ? " ↑"
                    : " ↓"
                  : ""}</th
              >
            {/each}
          </tr>
        </thead>
        <tbody>
          {#if tableCells}
            {#each tableCells as row}
              <tr>
                {#each chartData.axes as axis}
                  <td class="p-2 border border-black">
                    {row[axis.axis.axis_id]}
                  </td>
                {/each}
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    {/if}
  </div>
</div>

<script lang="ts">
  import { getChartData } from "$lib/api";
  import {
    AxisKind,
    ChartKind,
    type Chart,
    type ChartData,
    type Dashboard,
    type Cells,
    type FieldKind,
    type AxisField,
    FieldType,
    type Axis,
  } from "$lib/types.d.js";
  import { Chart as ChartGraphic, type ChartTypeRegistry } from "chart.js/auto";
  import { onMount } from "svelte";
  let { dashboard, chart }: { dashboard: Dashboard; chart: Chart } = $props();
  const SORTABLE_FIELDS = [
    FieldType.Integer,
    FieldType.Decimal,
    FieldType.Progress,
    FieldType.DateTime,
  ];
  let chartData: ChartData | null = $state(null);
  $inspect(chartData);
  let error = $state("");

  let enterModal = $state(false);

  let g: any;
  $effect(() => {
    if (chartData) {
      if (chartData.chart.chart_kind === ChartKind.Table) {
      } else {
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

  let selectedColumn = $state({ axis_id: -1, ascending: true });
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
      : true,
  );
  let tableCells = $derived(chartData.cells.toSorted(sortingMethod));
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
  class={enterModal
    ? "z-10 size-full fixed top-0 left-0 bg-black/25 flex justify-center items-center"
    : ""}
  onclick={() => {
    enterModal = false;
  }}
>
  <div
    onclick={(e) => {
      e.stopPropagation();
      enterModal = true;
    }}
    class={enterModal
      ? "bg-white rounded-lg p-3 size-1/2 transition-all"
      : "transition-all"}
  >
    {#if error}
      <p class="text-red-500">({error})</p>
    {:else if chartData && !(chartData.chart.chart_kind === ChartKind.Table)}
      <div class="size-full flex justify-center items-center">
        <canvas bind:this={g}></canvas>
      </div>
    {:else if chartData}
      <table class="border border-black">
        <thead>
          <tr>
            {#each chartData.axes as axis}
              <th
                class="border border-black bg-white select-none"
                onclick={() => {
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
          {#each tableCells as row}
            <tr>
              {#each chartData.axes as axis}
                <td class="p-2 border border-black">
                  {row[axis.axis.axis_id]}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>

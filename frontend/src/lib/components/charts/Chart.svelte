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
        $inspect(
          xAxis,
          yAxis,
          chartData.cells.map((row: Cells) => row[yAxis.axis.axis_id]),
        );

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

  onMount(() => {
    getChartData(dashboard, chart)
      .then((r: ChartData) => {
        chartData = r;
      })
      .catch((e) => {
        error = e.body.toString();
      });
  });
</script>

{#if error}
  <p class="text-red-500">({error})</p>
{:else if chartData && !(chartData.chart.chart_kind === ChartKind.Table)}
  <div>
    <canvas bind:this={g}></canvas>
  </div>
{:else if chartData}
  {#each chartData.axes as axis}
    <p>{axis.axis.axis_kind}: {axis.field_name}</p>
  {/each}
{/if}

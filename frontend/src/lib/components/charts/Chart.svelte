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
  } from "$lib/types.d.js";
  import { Chart as ChartGraphic } from "chart.js/auto";
  import { onMount } from "svelte";
  let { dashboard, chart }: { dashboard: Dashboard; chart: Chart } = $props();

  let chartData: ChartData | null = $state(null);
  let error = $state("");

  let g: any;
  $effect(() => {
    if (chartData) {
      if (chartData.chart.chart_kind === ChartKind.Bar) {
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
        new ChartGraphic(g, {
          type: "bar",
          data: {
            labels: chartData.cells.map(
              (row: Cells) => row[xAxis.axis.field_id ?? -1]?.toString() ?? "",
            ),
            datasets: [
              {
                data: chartData.cells.map(
                  (row: Cells) => row[yAxis.axis.field_id],
                ),
                backgroundColor: colorAxis
                  ? chartData.cells.map((row: Cells) => `rgba(0.2)`)
                  : undefined,
              },
            ],
          },
          options: {
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
          },
        });
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

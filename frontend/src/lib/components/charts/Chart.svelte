<script lang="ts">
  import { getChartData } from "$lib/api";
  import {
    AxisKind,
    ChartKind,
    type Chart,
    type ChartData,
    type Dashboard,
  } from "$lib/types.d.js";
  import { Chart as ChartGraphic } from "chart.js/auto";
  import { onMount } from "svelte";
  let { dashboard, chart }: { dashboard: Dashboard; chart: Chart } = $props();

  let chartData: ChartData | null = $state(null);
  let error = $state("");

  let g: any;
  $effect(() => {});

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
{:else if chartData && !(chartData.chart.chart_kind === ChartKind.Table) && false}
  <div bind:this={g}></div>
{:else if chartData}
  {#each chartData.axes as axis}
    <p>{axis.axis.axis_kind}: {axis.field_name}</p>
  {/each}
{/if}

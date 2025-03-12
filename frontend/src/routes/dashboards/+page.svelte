<script lang="ts">
  import D3Chart from "$lib/components/charts/D3Chart.svelte";
  import ChartJsChart from "$lib/components/charts/ChartJSChart.svelte";
</script>

<h1>This is the dashboard page!</h1>
<div class="flex">
  <D3Chart w={300} h={300} />
  <ChartJsChart w={300} h={300} />
</div>

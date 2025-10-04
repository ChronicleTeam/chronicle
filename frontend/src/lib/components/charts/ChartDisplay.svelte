<script lang="ts">
  import {
    AxisKind,
    ChartKind,
    type Chart as ChartType,
    type ChartData,
    type Dashboard,
    type Cells,
    type AxisField,
    FieldType,
  } from "$lib/types";
  let { dashboard, chart }: { dashboard: Dashboard; chart: ChartType } =
    $props();
  import Chart from "./Chart.svelte";
  let chartModal: HTMLDialogElement;

  const openModal = () => {
    chartModal?.showModal();
  };

  const closeModal = () => {
    chartModal?.close();
  };
</script>

<button class="flex justify-center" onclick={openModal}>
  <Chart {dashboard} {chart} />
</button>

<dialog bind:this={chartModal} class="modal">
  <div class="modal-box">
    <Chart {dashboard} {chart} />
  </div>
  <form method="dialog" class="modal-backdrop">
    <button>close</button>
  </form>
</dialog>

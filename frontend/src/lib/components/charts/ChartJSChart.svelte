<script lang="ts">
  import { Chart } from "chart.js/auto";
  let { w, h }: { w: number; h: number } = $props();

  // draw graph
  const placeholder = {
    A: 10,
    B: 20,
    C: 15,
  } as {
    [key: string]: number;
  };

  let g: any;

  $effect(() => {
    new Chart(g, {
      type: "bar",
      data: {
        labels: Object.entries(placeholder),
        datasets: [
          {
            label: "Height",
            data: Object.values(placeholder),
          },
        ],
      },
    });
  });
</script>

<p>Chart.js Graph:</p>
<div class="w-[{w}px] h-[{h}px]">
  <canvas bind:this={g}></canvas>
</div>

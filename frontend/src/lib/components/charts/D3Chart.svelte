<script lang="ts">
  import * as d3 from "d3";
  let { w, h }: { w: number; h: number } = $props();

  // draw graph
  const placeholder = {
    A: 10,
    B: 20,
    C: 15,
  } as {
    [key: string]: number;
  };

  // graphical x and y axes (what gets rendered)
  let gx: any;
  let gy: any;

  // x axis
  let x = $derived(
    d3
      .scaleBand()
      .domain(Object.keys(placeholder))
      .range([40, w - 40]),
  );

  // y axis
  let y = $derived(
    d3
      .scaleLinear()
      .domain([0, 30])
      .range([h - 40, 40]),
  );

  // on state change
  $effect(() => {
    // select gx and gy (the g elements) and render the bottom axis and left axis on them respectively (using the x and y scales defined earlier)
    d3.select(gx).call(d3.axisBottom(x));
    d3.select(gy).call(d3.axisLeft(y));
  });
</script>

<p>D3 Graph:</p>
<svg width={w} height={h}>
  <g bind:this={gy} transform="translate(40, 0)" />
  <g bind:this={gx} transform="translate(0, {h - 40})" />
  <g fill="blue" stroke="black" fill-opacity="0.5">
    {#each Object.entries(placeholder) as datum}
      <!-- x(l) gives the x position of the band with label l -->
      <rect
        x={(x(datum[0]) as number) + 5}
        y={y(datum[1])}
        width={x.bandwidth() - 10}
        height={y(0) - y(datum[1])}
      />
    {/each}
  </g>
</svg>

<script lang="ts">
import { type InputParameters } from "$lib/types.d.js";

let { params, disabled = false, id, innerClass = "" }: { params: InputParameters, disabled?: boolean, id?: string, innerClass: any } = $props();

</script>

{#if params.label ?? false}
  <label class={["mr-2 min-w-28", disabled && "text-gray-300"]} for={id}>{params.label}:</label>
{/if}
{#if params.type === "select"}
  <select disabled={disabled} id={id} bind:value={params.bindGetter, params.bindSetter}>
    {#each params.selectOptions as opt}
      <option>{opt}</option>
    {/each}
  </select>
{:else if params.type === "checkbox"}
  <input disabled={disabled} id={id} type="checkbox" bind:checked={params.bindGetter, params.bindSetter} />
{:else}
  <input disabled={disabled} id={id} class={innerClass} type={params.type} bind:value={params.bindGetter, params.bindSetter} />
{/if}

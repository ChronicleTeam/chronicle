<script lang="ts">
  import { type InputParameters } from "$lib/types";
  import type { ClassValue } from "svelte/elements";

  let {
    params, // Controls input type, certain attributes, and behaviour
    disabled = false, // maps to input disabled attribute
    id, // maps to id attribute
    class: innerClass = "", // maps to class attribute
    onclick, // maps to onclick attribute
    onkeydown,
  }: {
    params: InputParameters;
    disabled?: boolean;
    id?: string;
    class?: ClassValue;
    onclick?: () => void;
    onkeydown?: (e: KeyboardEvent) => void;
  } = $props();
</script>

<div class={params.label ? "flex justify-between w-full" : ""}>
  {#if params.label ?? false}
    <label class={["mr-2 w-auto", disabled && "text-gray-300"]} for={id}
      >{params.label}:</label
    >
  {/if}
  {#if params.type === "select"}
    {@const opts = Array.isArray(params.selectOptions)
      ? params.selectOptions.map((o) => [o, o])
      : Object.entries(params.selectOptions)}
    <select
      {disabled}
      {id}
      bind:value={params.bindGetter, params.bindSetter}
      {onclick}
      {onkeydown}
    >
      {#each opts as opt}
        <option value={opt[0]}>{opt[1]}</option>
      {/each}
    </select>
  {:else if params.type === "textarea"}
    <textarea
      {disabled}
      {id}
      class={innerClass}
      bind:value={params.bindGetter, params.bindSetter}
      {onclick}
      {onkeydown}
    ></textarea>
  {:else if params.type === "checkbox"}
    <input
      {disabled}
      {id}
      type="checkbox"
      bind:checked={params.bindGetter, params.bindSetter}
      {onclick}
      {onkeydown}
    />
  {:else if params.type === "url"}
    {#if disabled}
      <a
        href={"https://" + params.bindGetter().toString()}
        class={["text-blue-600 underline cursor-pointer", innerClass]}
        target="_blank">{params.bindGetter().toString()}</a
      >
    {:else}
      <input
        {disabled}
        {id}
        class={innerClass}
        type={params.type}
        bind:value={params.bindGetter, params.bindSetter}
        {onclick}
        {onkeydown}
      />
    {/if}
  {:else}
    <input
      {disabled}
      {id}
      class={innerClass}
      type={params.type}
      min={params.min instanceof Date
        ? params.min.toISOString().substring(0, 19)
        : params.min}
      max={params.max instanceof Date
        ? params.max.toISOString().substring(0, 19)
        : params.max}
      step={params.step instanceof Date
        ? params.step.toISOString().substring(0, 19)
        : params.step}
      bind:value={params.bindGetter, params.bindSetter}
      {onclick}
      {onkeydown}
    />
  {/if}
</div>

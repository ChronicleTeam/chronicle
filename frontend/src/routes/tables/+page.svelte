<script lang="ts">
  import type { PageProps } from './$types';
  import DataTable from './DataTable.svelte';
  type Id = number;

  type Table = {
    table_id: Id;
    user_id: Id;
    name: string;
    description: string;
    created_at: Date;
    updated_at?: Date;
  }

  let { data }: PageProps = $props();

  let curTable = $state(null as unknown as Table)
</script>


<div class="flex gap-4 p-4 size-full">
  <div class="basis-1/3 bg-gray-200 rounded-lg p-3 size-full">
    <h2>Tables</h2>
    <div class="flex flex-col">
      {#await data.tables}
        Loading...
      {:then tables}
        {#each tables as t}
          <button onclick={() => {curTable = t}} 
            class="text-left bg-gray-200 hover:bg-gray-400 transition duration-300 rounded-md p-2">{t.name}</button>
        {/each}
      {/await}
    </div>
    <button class="text-center w-full rounded-xl p-2 hover:p-3 transition-size duration-300 border-2 border-dashed border-black">Add Table</button>
  </div>
   <div class="bg-gray-200 rounded-lg p-3 size-full flex flex-col items-center">
    {#if curTable === null}
      <h2 class="text-lg font-bold">Select a Table</h2>
    {:else}
      <div class="flex items-center gap-2 mb-2">
        <h2 class="text-lg font-bold">{curTable.name}</h2>
        <button class="px-2 bg-white hover:bg-gray-100 transition duration-300 rounded-md">Edit</button>
        <button class="px-2 bg-red-400 hover:bg-red-500 transition duration-300 rounded-md">Delete</button>
      </div>
      <DataTable table_prop={curTable}/>
    {/if}
  </div>
</div>

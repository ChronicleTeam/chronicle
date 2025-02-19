<script lang="ts">
  import type { PageProps } from './$types';
  import DataTable from './DataTable.svelte';
  import FieldEditor from './FieldEditor.svelte';
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

  const EditMode = {
    NONE: 0,
    TABLE: 1,
    FIELDS: 2
  }

  let editMode = $state(EditMode.NONE)
</script>


<div class="flex gap-4 p-4 size-full items-stretch">

  <!-- Sidebar -->
  <div class="basis-1/5 bg-gray-200 rounded-lg p-3">
    <h2>Tables</h2>
    <div class="flex flex-col">
      {#await data.tables}
        Loading...
      {:then tables}
        {#each tables as t}
          <button onclick={() => {curTable = t; editMode = EditMode.TABLE}} 
            class="text-left bg-gray-200 hover:bg-gray-400 transition duration-300 rounded-md p-2">{t.name}</button>
        {/each}
      {/await}
    </div>
    <button class="text-center w-full rounded-xl p-2 hover:p-3 transition-size duration-300 border-2 border-dashed border-black">Add Table</button>
  </div>

  <!-- Main Editor -->
  <div class="bg-gray-200 basis-4/5 rounded-lg p-3 w-full flex flex-col items-center flex-none">
    {#if editMode === EditMode.NONE}
      <h2 class="text-lg font-bold">Select a Table</h2>
    {:else if editMode === EditMode.TABLE}
      <!-- Top Bar -->
      <div class="flex items-center gap-2 mb-2">
        <h2 class="text-lg font-bold">{curTable.name}</h2>
        <button onclick={()=>{editMode = EditMode.FIELDS}} class="px-2 bg-white hover:bg-gray-100 transition duration-300 rounded-md">Edit</button>
        <button class="px-2 bg-red-400 hover:bg-red-500 transition duration-300 rounded-md">Delete</button>
      </div>

      <!-- Main Table -->
      <DataTable table_prop={curTable}/>
    {:else if editMode === EditMode.FIELDS}
      <FieldEditor table_prop={curTable} />
    {/if}
  </div>
</div>

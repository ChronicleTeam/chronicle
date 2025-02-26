<script lang="ts">
  import type { PageProps } from './$types';
  import DataTable from './DataTable.svelte';
  import FieldEditor from './FieldEditor.svelte';
  import { API_URL } from '$lib/api.d.js';
  type Id = number;

  type Table = {
    table_id: Id;
    user_id: Id;
    name: string;
    description: string;
    created_at: Date;
    updated_at?: Date;
  }



  const loadTables: () =>Promise<Table[]> = () => fetch(API_URL + "/tables").then(response => response.json())

  const addTable = () => {
      fetch(API_URL + "/tables", {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          name: "Table 1" + Math.floor(Math.random() * 10000),
          description: ""
        })
      }).then(() => {
          asyncTables = loadTables();
        })
  };

  let asyncTables: Promise<Table[]> = $state(loadTables())

  let curTable = $state(null as unknown as Table)

  const EditMode = {
    NONE: 0,
    TABLE: 1,
    FIELDS: 2
  }

  let editMode = $state(EditMode.NONE)
</script>


<div class="flex flex-wrap gap-4 p-4 size-full items-stretch">

  <!-- Sidebar -->
  <div class="basis-[12rem] grow bg-gray-200 rounded-lg p-3">
    <h2>Tables</h2>
    <div class="flex flex-col">
      {#await asyncTables}
        Loading...
      {:then tables}
        {#each tables as t}
          <button onclick={() => {curTable = t; editMode = EditMode.TABLE}} 
            class="text-left bg-gray-200 hover:bg-gray-400 transition duration-300 rounded-md p-2">{t.name}</button>
        {/each}
      {/await}
    </div>
    <button onclick={addTable} class="text-center w-full rounded-xl p-2 border-2 border-dashed border-gray-400 hover:bg-gray-400 transition-all">Add Table</button>
  </div>

  <!-- Main Editor -->
  <div class="bg-gray-200 basis-[36rem] grow-[5] shrink min-w-0 rounded-lg p-3 flex flex-col items-center ">
    {#if editMode === EditMode.NONE}
      <h2 class="text-lg font-bold">Select a Table</h2>
    {:else if editMode === EditMode.TABLE}
      <!-- Top Bar -->
      <div class="flex items-center gap-2 mb-2">
        <h2 class="text-lg font-bold">{curTable.name}</h2>
        <button onclick={()=>{editMode = EditMode.FIELDS}} class="px-2 bg-white hover:bg-gray-100 transition rounded">Edit</button>
        <button class="px-2 bg-red-400 hover:bg-red-500 transition rounded">Delete</button>
      </div>

      <!-- Main Table -->
      <DataTable table_prop={curTable}/>
    {:else if editMode === EditMode.FIELDS}
      <FieldEditor table_prop={curTable} />
    {/if}
  </div>
</div>

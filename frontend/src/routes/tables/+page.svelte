<script lang="ts">
  import type { Table } from "$lib/types.d.js";
  import DataTable from "./DataTable.svelte";
  import FieldEditor from "./FieldEditor.svelte";
  import {
    getTables,
    postTable,
    deleteTable,
    type APIError,
  } from "$lib/api.js";

  //
  // Constants
  //

  const EditMode = {
    NONE: 0,
    TABLE: 1,
    FIELDS: 2,
  };

  //
  // State
  //

  // currently selected table
  let curTable: Table | null = $state(null as unknown as Table);

  // current editing mode: whether for the datatable or the fields (or neither, if no table is selected)
  let editMode = $state(EditMode.NONE);

  // for the table creation input
  let addTableMode = $state(false);
  let addTableField = $state("");

  //
  // API Calls
  //

  let asyncTables: Promise<Table[]> = $state(getTables());

  let addTableError = $state("");
  const addTable = (name: string) =>
    postTable(name)
      .then(() => {
        addTableMode = false;
        asyncTables = getTables();
        addTableError = "";
        addTableField = "";
      })
      .catch((e: APIError) => {
        addTableError = "Error: " + (e.body as { [key: string]: string }).name;
      });

  let deleteTableError = $state("");
  const deleteCurTable = () => {
    if (curTable === null) {
      return;
    }

    deleteTable(curTable)
      .then(() => {
        curTable = null;
        asyncTables = getTables();
        deleteTableError = "";
        editMode = EditMode.NONE;
      })
      .catch(() => {
        deleteTableError = "An error occured.";
      });
  };
</script>

<div class="flex flex-wrap gap-4 p-4 size-full items-stretch">
  <!-- Sidebar -->
  <div class="basis-[12rem] grow bg-gray-200 rounded-lg p-3">
    <!-- Table list -->
    <h2>Tables</h2>
    <div class="flex flex-col">
      {#await asyncTables}
        Loading...
      {:then tables}
        {#each tables as t}
          <button
            onclick={() => {
              curTable = t;
              editMode = EditMode.TABLE;
            }}
            class="text-left bg-gray-200 hover:bg-gray-400 transition rounded-md p-2"
            >{t.name}</button
          >
        {/each}
      {/await}
    </div>
    <!-- Table creation input -->
    <div
      class={[
        "rounded-xl py-2 border-2 border-dashed border-gray-400 flex flex-col items-center transition gap-3",
        !addTableMode && "hover:bg-gray-400",
      ]}
    >
      {#if addTableMode}
        <p class="text-center">New Table</p>
        <input bind:value={addTableField} id="table-name-input" />

        <div class="flex gap-3">
          <button
            onclick={() => addTable(addTableField)}
            class="px-2 py-1 rounded-lg border-2 border-gray-400 hover:bg-gray-400 transition"
            >Create</button
          >

          <button
            onclick={() => {
              addTableError = "";
              addTableField = "";
              addTableMode = false;
            }}
            class="px-2 py-1 rounded-lg border-2 border-red-400 hover:bg-red-400 transition"
            >Cancel</button
          >
        </div>
      {:else}
        <button
          onclick={() => {
            addTableMode = true;
          }}
          class="text-center w-full">Add Table</button
        >
      {/if}
      {#if addTableError !== ""}
        <p class="text-red-500">{addTableError}</p>
      {/if}
    </div>
  </div>
  <!-- Main editor -->
  <div
    class="bg-gray-200 basis-[36rem] grow-[5] shrink min-w-0 rounded-lg p-3 flex flex-col items-center"
  >
    {#if editMode === EditMode.NONE || curTable === null}
      <h2 class="text-lg font-bold">Select a Table</h2>
    {:else if editMode === EditMode.TABLE && curTable !== null}
      <!-- Top bar -->
      <div class="flex items-center gap-2">
        <h2 class="text-lg font-bold">{curTable.name}</h2>
        <button
          onclick={() => {
            editMode = EditMode.FIELDS;
          }}
          class="px-2 bg-white hover:bg-gray-100 transition rounded"
          >Edit</button
        >
      </div>
      <h3 class="text-lg mb-2">{curTable.description}</h3>

      <!-- Main Table -->
      {#key curTable.table_id}
        <DataTable table_prop={curTable} />
      {/key}
      <!-- Error -->
      {#if deleteTableError !== ""}
        <p class="text-red-500">{deleteTableError}</p>
      {/if}
    {:else if editMode === EditMode.FIELDS && curTable !== null}
      <!-- Field editor -->
      <FieldEditor
        on_save={() => {
          editMode = EditMode.TABLE;
          asyncTables = getTables();
        }}
        delete_table={deleteCurTable}
        table_prop={curTable}
      />
    {/if}
  </div>
</div>

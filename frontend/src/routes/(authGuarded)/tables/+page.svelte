<script lang="ts">
  import type { Table } from "$lib/types.d.js";
  import TableEditor from "./TableEditor.svelte";
  import FieldEditor from "./FieldEditor.svelte";
  import {
    getTables,
    postCreateTable,
    deleteTable,
    type APIError,
    postImportTable,
  } from "$lib/api";

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
  let importTableFiles: FileList | null = $state(null);

  let errors = $state({
    table: {
      add: "",
      delete: "",
    },
  });

  //
  // API Calls
  //

  let asyncTables: Promise<Table[]> = $state(getTables());

  const afterTableCreation = () => {
    addTableMode = false;
    asyncTables = getTables();
    errors.table.add = "";
    addTableField = "";
  };

  const addTable = (name: string) =>
    postCreateTable({ name, description: "", table_id: -1, user_id: -1 })
      .then(afterTableCreation)
      .catch((e: APIError) => {
        errors.table.add =
          "Error: " + (e.body as { [key: string]: string }).name;
      });

  const importTable = (file: File) =>
    postImportTable(file)
      .then(afterTableCreation)
      .catch((e) => {
        errors.table.add =
          "Error: " + (e.body as { [key: string]: string }).name;
      });

  const deleteCurTable = () => {
    if (curTable === null) {
      return;
    }

    deleteTable(curTable)
      .then(() => {
        curTable = null;
        asyncTables = getTables();
        errors.table.delete = "";
        editMode = EditMode.NONE;
      })
      .catch(() => {
        errors.table.delete = "An error occured.";
      });
  };
</script>

<div class="flex flex-wrap gap-4 size-full items-stretch">
  <!-- Sidebar -->
  <div class="basis-[12rem] grow bg-gray-200 rounded-lg p-3">
    <!-- Table list -->
    <h2>Tables</h2>
    <div class="flex flex-col">
      {#await asyncTables}
        Loading...
      {:then tables}
        {#each tables.filter((t) => t.parent_id == null) as t}
          <button
            onclick={() => {
              curTable = t;
              editMode = EditMode.TABLE;
            }}
            class="text-left bg-gray-200 hover:bg-gray-400 transition rounded-xl p-2 mb-2"
            >{t.name}</button
          >
        {/each}
      {/await}
    </div>
    <!-- Table creation input -->
    <div
      class={[
        "rounded-xl p-2 border-2 border-dashed border-gray-400 flex flex-col items-center transition gap-3",
        !addTableMode && "hover:bg-gray-400",
      ]}
    >
      {#if addTableMode}
        <p class="text-center font-bold">Create New Table:</p>
        <div class="flex gap-2 items-center">
          <input bind:value={addTableField} id="table-name-input" />
          <button
            onclick={() => addTable(addTableField)}
            class="px-2 py-1 rounded-lg border-2 border-gray-400 hover:bg-gray-400 transition"
            >Create</button
          >
        </div>
        <p class="text-center">or</p>
        <p class="font-bold">Import existing table:</p>
        <div class="flex gap-2 items-center">
          <input
            type="file"
            accept=".xlsx,.csv"
            bind:files={importTableFiles}
          />
          <button
            class="px-2 py-1 rounded-lg border-2 border-gray-400 hover:bg-gray-400 transition"
            onclick={() =>
              importTableFiles ? importTable(importTableFiles[0]) : null}
            disabled={!importTableFiles}>Import</button
          >
        </div>

        <div class="flex gap-3">
          <button
            onclick={() => {
              errors.table.add = "";
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
      {#if errors.table.add !== ""}
        <p class="text-red-500">{errors.table.add}</p>
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
        <TableEditor table_prop={curTable} />
      {/key}
      <!-- Error -->
      {#if errors.table.delete !== ""}
        <p class="text-red-500">{errors.table.delete}</p>
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

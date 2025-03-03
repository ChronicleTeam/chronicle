<script lang="ts">
  import type { PageProps } from "./$types";
  import DataTable from "./DataTable.svelte";
  import FieldEditor from "./FieldEditor.svelte";
  import { API_URL } from "$lib/api.d.js";
  type Id = number;

  type Table = {
    table_id: Id;
    user_id: Id;
    name: string;
    description: string;
    created_at: Date;
    updated_at?: Date;
  };

  const loadTables: () => Promise<Table[]> = () =>
    fetch(`${API_URL}/tables`).then((response) => response.json());

  const addTable = (name: string) => {
    fetch(API_URL + "/tables", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        name,
        description: "",
      }),
    }).then((r) => {
      if (r.ok) {
        addTableMode = false;
        asyncTables = loadTables();
        addTableError = "";
      } else {
        if (r.headers.get("content-type") === "application/json") {
          r.json().then((j) => {
            addTableError = "Error: " + j.name;
          });
        }
        addTableError = r.statusText;
      }
    });
  };

  let asyncTables: Promise<Table[]> = $state(loadTables());

  let curTable: Table | null = $state(null as unknown as Table);

  const EditMode = {
    NONE: 0,
    TABLE: 1,
    FIELDS: 2,
  };

  let editMode = $state(EditMode.NONE);

  let addTableMode = $state(false);
  let addTableField = $state("");
  let addTableError = $state("");

  const deleteCurTable = () => {
    if (curTable === null) {
      return;
    }

    fetch(`${API_URL}/tables/${curTable.table_id}`, {
      method: "DELETE",
    }).then((r) => {
      if (r.ok) {
        curTable = null;
        asyncTables = loadTables();
        deleteTableError = "";
        editMode = EditMode.NONE;
      } else {
        deleteTableError = "An error occured.";
      }
    });
  };

  let deleteTableError = $state("");
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
    <div
      class={[
        "rounded-xl py-2 border-2 border-dashed border-gray-400 flex flex-col items-center transition gap-3",
        !addTableMode && "hover:bg-gray-400",
      ]}
    >
      {#if addTableMode}
        <p class="text-center">New Table</p>
        <input bind:value={addTableField} id="table-name-input" />
        <button
          onclick={() => addTable(addTableField)}
          class="px-2 py-1 rounded-lg border-2 border-gray-400 hover:bg-gray-400 transition"
          >Create</button
        >
      {:else}
        <button
          onclick={() => {
            addTableMode = true;
          }}
          class="text-center w-full">Add Table</button
        >
      {/if}
    </div>
    {#if addTableError !== ""}
      <p class="text-red-500">{addTableError}</p>
    {/if}
  </div>
  <!-- Main Editor -->
  <div
    class="bg-gray-200 basis-[36rem] grow-[5] shrink min-w-0 rounded-lg p-3 flex flex-col items-center"
  >
    {#if editMode === EditMode.NONE || curTable === null}
      <h2 class="text-lg font-bold">Select a Table</h2>
    {:else if editMode === EditMode.TABLE && curTable !== null}
      <!-- Top Bar -->
      <div class="flex items-center gap-2">
        <h2 class="text-lg font-bold">{curTable.name}</h2>
        <button
          onclick={() => {
            editMode = EditMode.FIELDS;
          }}
          class="px-2 bg-white hover:bg-gray-100 transition rounded"
          >Edit</button
        >
        <button
          onclick={deleteCurTable}
          class="px-2 bg-red-400 hover:bg-red-500 transition rounded"
          >Delete</button
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
      <FieldEditor
        on_save={() => {
          editMode = EditMode.TABLE;
          asyncTables = loadTables();
        }}
        table_prop={curTable}
      />
    {/if}
  </div>
</div>

<script lang="ts">
  import type { Table } from "$lib/types";
  import TableEditor from "./TableEditor.svelte";
  import FieldEditor from "./FieldEditor.svelte";
  import {
    getTables,
    postCreateTable,
    deleteTable,
    type APIError,
    postImportTable,
    postExportTable,
  } from "$lib/api";

  //
  // Constants
  //

  const EditMode = {
    NONE: 0,
    TABLE: 1,
    FIELDS: 2,
  };

  const FileTypes = {
    csv: "text/csv",
    excel: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  };

  const FileExtensions = {
    csv: ".csv",
    excel: ".xlsx",
  };

  //
  // State
  //

  // currently selected table
  let curTable: Table | null = $state({} as unknown as Table);

  // current editing mode: whether for the datatable or the fields (or neither, if no table is selected)
  let editMode = $state(EditMode.TABLE);

  // for the table creation input
  let addTableField = $state("");
  let importTableFiles: FileList | null = $state(null);

  let errors = $state({
    table: {
      add: "",
      delete: "",
      export: "",
    },
  });

  //
  // API Calls
  //

  let asyncTables: Promise<Table[]> = $state(getTables());

  const afterTableCreation = () => {
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

  const exportTable = (type: "csv" | "excel") => {
    if (curTable) {
      let t = curTable;
      postExportTable(curTable, type)
        .then((r) => {
          let exportedFile = new File(
            [r],
            t.name.replaceAll(" ", "_") + FileExtensions[type],
            {
              type: FileTypes[type],
            },
          );
          open(URL.createObjectURL(exportedFile));
        })
        .catch((e) => {
          errors.table.export = e.body.toString();
        });
    }
  };

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

<div class="flex flex-wrap gap-4 w-full grow items-stretch">
  <!-- Sidebar -->
  <div class="basis-48 grow bg-base-300 rounded-lg shadow-xs">
    <ul class="menu w-full">
      <!-- Table list -->
      <li class="menu-title">Tables</li>
      {#await asyncTables}
        Loading...
      {:then tables}
        {#each tables.filter((t) => t.parent_id == null) as t}
          <li>
            <button
              onclick={() => {
                curTable = t;
                editMode = EditMode.TABLE;
              }}
              class={{ "menu-active": curTable?.table_id === t.table_id }}
              >{t.name}</button
            >
          </li>
        {/each}
      {/await}
    </ul>
    <!-- Table creation input -->
    <div
      class="collapse collapse-plus bg-base-100 stroke-base-200 rounded-md mx-2 w-auto"
    >
      <input type="checkbox" />
      <div class="collapse-title text-sm">Add Table</div>
      <div class="collapse-content flex flex-col">
        <!-- Create new table -->
        <p class="self-start font-semibold mb-4">Create new table</p>
        <div class="join">
          <input
            class="input join-item w-full"
            placeholder="Table name"
            bind:value={addTableField}
            id="table-name-input"
          />
          <button onclick={() => addTable(addTableField)} class="btn join-item"
            >Create</button
          >
        </div>
        <div class="divider">or</div>

        <!-- Import from csv or excel -->
        <p class="self-start font-semibold mb-4">Import existing table</p>
        <div class="join">
          <input
            class="file-input join-item"
            type="file"
            accept=".xlsx,.csv"
            bind:files={importTableFiles}
          />
          <button
            class="btn join-item"
            onclick={() =>
              importTableFiles ? importTable(importTableFiles[0]) : null}
            disabled={!importTableFiles}>Import</button
          >
        </div>
        {#if errors.table.add !== ""}
          <p class="text-error">{errors.table.add}</p>
        {/if}
      </div>
    </div>
  </div>
  <!-- Main editor -->
  <div
    class="bg-base-300 basis-xl grow-5 shrink min-w-0 rounded-lg p-3 flex flex-col items-center"
  >
    {#if editMode === EditMode.NONE || curTable === null}
      <h2 class="text-xl font-bold">Select a Table</h2>
    {:else if editMode === EditMode.TABLE && curTable !== null}
      <!-- Top bar -->
      <div class="flex justify-between items-center gap-2">
        <div></div>
        <h2 class="text-lg font-bold">{curTable.name}</h2>
        <div>
          <button
            onclick={() => {
              editMode = EditMode.FIELDS;
            }}
            class="btn">Edit</button
          >
          <!-- Export buttons -->
          <details class="dropdown">
            <summary class="btn">Export</summary>
            <ul class="dropdown-content menu bg-base-100 rounded-md m-1 w-64">
              <li>
                <button onclick={() => exportTable("csv")}>Export as CSV</button
                >
              </li>
              <li>
                <button onclick={() => exportTable("excel")}
                  >Export as Excel Spreadsheet</button
                >
              </li>
            </ul>
          </details>
        </div>
      </div>

      {#if errors.table.export}
        <p class="text-error">Could not export table.</p>
      {/if}
      <h3 class="text-lg mb-2">{curTable.description}</h3>

      <!-- Main Table -->
      {#key curTable.table_id}
        <TableEditor table_prop={curTable} />
      {/key}
      <!-- Error -->
      {#if errors.table.delete !== ""}
        <p class="text-error">{errors.table.delete}</p>
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

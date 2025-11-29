<script lang="ts">
  import type { Table } from "$lib/types";
  import { postCreateTable, type APIError, postImportTable } from "$lib/api";

  let { children, data } = $props();
  import { page } from "$app/state";
  import { invalidateAll } from "$app/navigation";

  //
  // State
  //

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

  let tables: Table[] = $derived(data.tables.map((result) => result.table));
  let subtable = $derived(data.subtable?.table_data);

  const afterTableCreation = async () => {
    await invalidateAll();
    errors.table.add = "";
    addTableField = "";
  };

  const addTable = (name: string) =>
    postCreateTable({ name, description: "", table_id: -1, user_id: -1 })
      .then(afterTableCreation)
      .catch((e: APIError) => {
        errors.table.add = "Error: " + e.body.toString();
      });

  const importTable = (file: File) =>
    postImportTable(file)
      .then(afterTableCreation)
      .catch((e) => {
        errors.table.add = "Error: " + e.body.toString();
      });
</script>

<div class="flex flex-wrap gap-4 w-full grow items-stretch">
  <!-- Sidebar -->
  <div class="basis-48 grow bg-base-300 rounded-lg shadow-xs">
    <ul class="menu w-full">
      <!-- Table list -->
      <li class="menu-title">Tables</li>
      {#each tables.filter((t) => t.parent_id == null) as t}
        <li>
          <a
            class={{
              "menu-active":
                t.table_id.toString() === page.params.table_id && !subtable,
            }}
            href={`/tables/${t.table_id}`}>{t.name}</a
          >
          {#if t.table_id.toString() === page.params.table_id && subtable}
            <ul>
              <li><p class="menu-active">{subtable.table.name}</p></li>
            </ul>
          {/if}
        </li>
      {/each}
    </ul>
    <!-- Table creation input -->
    <div
      class="collapse collapse-plus bg-base-100 stroke-base-200 rounded-md mx-2 w-auto"
    >
      <input type="checkbox" aria-label="add table" />
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
            aria-label="file input"
            class="file-input join-item"
            type="file"
            accept=".xlsx,.csv"
            bind:files={importTableFiles}
          />
          <button
            class="btn join-item"
            onclick={() => importTable((importTableFiles as FileList)[0])}
            disabled={!importTableFiles || importTableFiles.length === 0}
            >Import</button
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
    class="bg-base-300 shadow-xs basis-xl grow-5 shrink min-w-0 rounded-lg p-3 flex flex-col items-center"
  >
    {@render children()}
  </div>
</div>

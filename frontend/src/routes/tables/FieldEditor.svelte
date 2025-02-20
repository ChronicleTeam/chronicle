<script lang="ts">
  import type {
    DataTable,
    Field,
    Entry,
    Cell,
    Text,
    Money,
    Integer,
    Progress,
  } from "$lib/types.d.js";

  const fieldTypes = ["Text", "Money", "Integer", "Progress"];
  let { table_prop } = $props();

  let table: DataTable = {
    table: table_prop,
    fields: [
      {
        name: "Project Name",
        options: {
          type: "Text",
          is_required: true,
        },
      },
      {
        name: "Funding",
        options: {
          type: "Money",
          is_required: false,
        },
      },
      {
        name: "Members",
        options: {
          type: "Integer",
          is_required: true,
        },
      },
      {
        name: "Progress",
        options: { type: "Progress" },
      },
    ],
    entries: [],
  };
</script>

<div class="w-full">
  <!-- Top bar -->
  <input bind:value={table.table.name} class="text-lg font-bold mb-3" />
  <!-- Fields -->
  <div class="flex items-stretch gap-5 w-full flex-nowrap">
    {#each table.fields as field, i}
      <div class="bg-white p-3 rounded-lg">
        <input bind:value={table.fields[i].name} />
        <div class="flex items-center">
          <label for="typeSelect" class="mr-2">Type:</label>
          <select
            id="typeSelect"
            class="my-2"
            bind:value={table.fields[i].options.type}
          >
            {#each fieldTypes as fieldType}
              <option value={fieldType}>{fieldType}</option>
            {/each}
          </select>
        </div>
      </div>
    {/each}
  </div>
</div>

<script lang="ts">
  import {
    type DataTable,
    type Field,
    type Entry,
    type Cell,
    type Text,
    type Money,
    type Integer,
    type Progress,
    type TextOptions,
    type MoneyOptions,
    type IntegerOptions,
    type ProgressOptions,
    FieldType,
  } from "$lib/types.d.js";

  let { table_prop } = $props();

  const fieldTypes = Object.values(FieldType);
  let table: DataTable = {
    table: table_prop,
    fields: [
      {
        name: "Project Name",
        options: {
          type: FieldType.Text,
          is_required: true,
        },
      },
      {
        name: "Funding",
        options: {
          type: FieldType.Money,
          is_required: false,
        },
      },
      {
        name: "Members",
        options: {
          type: FieldType.Integer,
          is_required: true,
        },
      },
      {
        name: "Progress",
        options: { 
          type: FieldType.Progress, 
          total_steps: 100
        },
      },
    ],
    entries: [],
  };  

  // TODO: add setter functions for bind() call such that the field options change type when the field type is changed.
</script>

<div class="w-full">
  <!-- Top bar -->
  <input bind:value={table.table.name} class="text-lg font-bold mb-3" />
  <!-- Fields -->
  <div class="flex items-stretch gap-5 w-full flex-nowrap overflow-scroll">
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

<script lang="ts">
import type { DataTable, Field, Entry,  Cell, Text, Money, Integer, Progress } from "$lib/types.d.js";
import { FieldType } from "$lib/types.d.js"
import { API_URL } from "$lib/api.d.js";

let { table_prop } = $props();

let err = $state();

const loadTable = () => {
  fetch(`${API_URL}/tables/${table_prop.table_id}/data`)
    .then((response) => response.json())
    .then((json) => {table = json})
};

const EntryMode = {
  DISPLAY: 0,
  INSERT: 1,
}

let table = $state({
  table: table_prop,
  fields: [],
  entries: []
} as DataTable);

loadTable()


let entryMode = $state(EntryMode.DISPLAY);
let newEntry = $state(null as unknown as Entry);

// TODO: implement for all types
const getNewEntry = (): Entry => {
  return {
    cells: Object.fromEntries(table.fields.map((f: Field): [string, Cell] => {
      switch(f.options.type){
        case FieldType.Text:
          return [f.field_id.toString(), "" as Text];
        case FieldType.Money:
          return [f.field_id.toString(), 0 as Money];
        case FieldType.Integer:
          return [f.field_id.toString(), 0 as Integer];
        case FieldType.Progress:
          return [f.field_id.toString(), 0 as Progress];
        default:
          return [f.field_id.toString(), "" as Text];
      }
    }))
  };
};

const insertEntry = () => {
  entryMode = EntryMode.INSERT;
  newEntry = getNewEntry()
};

const saveEntry = () => {
  table.entries.push(newEntry);

  fetch(`${API_URL}/tables/${table_prop.table_id}/entries`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(newEntry.cells)
  }).then(cancelEntry).then(loadTable);
};

const cancelEntry = () => {
  entryMode = EntryMode.DISPLAY;
  newEntry = null as unknown as Entry;
};

$inspect(table, newEntry);
</script>
<div class="flex flex-col items-center justify-center gap-3">
    <table class=" border border-gray-400 bg-white text-black w-full">
      <thead>
        <tr>
        {#each table.fields as field}
          <th class="bg-gray-200 p-1 border-2 border-gray-400">{field.name}</th>
        {/each}
        </tr>
      </thead>
      <tbody>
        {#each table.entries as entry}
          <tr>
            {#each table.fields as field}
              <td class="border-2 border-gray-400 bg-white p-1 ">{entry.cells[field.field_id]}</td>
            {/each}
          </tr>
        {/each}
        {#if entryMode === EntryMode.INSERT}
          <tr>
            {#if newEntry === null}
              <td>error</td>
            {:else}
              {#each Object.keys(newEntry.cells) as key }
                <td class="text-gray-500 border-2 border-gray-400 bg-white size-min">
                  <input bind:value={newEntry.cells[key]} class="border-none focus:outline-hidden outline-none size-full"/>
                </td>
              {/each}
            {/if}
          </tr>
        {/if}
      </tbody>
    </table> 
    {#if entryMode === EntryMode.INSERT}
      <div class="flex justify-center gap-3">
        <button onclick={saveEntry} class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition">Save</button>
        <button onclick={cancelEntry} class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition">Cancel</button>
      </div>
    {:else if table.fields.length > 0}
      <button onclick={insertEntry} class="text-center w-full mt-1 py-1 border-2 border-dashed border-gray-400 hover:bg-gray-400 transition">+ Add Row</button>
    {/if}
{err}
</div>

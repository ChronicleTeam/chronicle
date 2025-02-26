<script lang="ts">
import type { DataTable, Field, Entry,  Cell, Text, Money, Integer, Progress } from "$lib/types.d.js";
import {FieldType } from "$lib/types.d.js"
import { API_URL } from "$lib/api.d.js";

let { table_prop } = $props();

let err = $state();

const loadTable = fetch(API_URL + `/tables/${table_prop.table_id}/data`)
    .then((response) => response.json())
    .then((json) => {table = json})
    .catch((e) => {
      err = e;
      table = {
      table: table_prop,
      fields: [
        {
          name: "Project Name",
          options: {
              type: FieldType.Text,
              is_required: true
            }
        },
        {
          name: "Funding",
          options: {
            type: FieldType.Money,
            is_required: false
          }
        },
        {
          name: "Members",
          options: {
            type: FieldType.Integer,
            is_required: true
          }
        },
        {
    
          name: "Progress",
          options: {
            type: FieldType.Progress,
            total_steps: 100
          }
        }
      ],
    
      entries: [
        {
          cells: [
            "Project Alpha" as Text,
            30000 as Money,
            1 as Integer,
            60 as Progress
          ]
        },
        {
            cells: [
            "Project Beta" as Text,
            40000 as Money,
            2 as Integer,
            20 as Progress
          ]
        }
      ]
    }
  });






let table = $state({
  table: table_prop,
  fields: [],
  entries: []
} as DataTable);



let insertEntryMode = $state(false);
let newEntry = $state(null as unknown as Entry);

const getNewEntry = (): Entry => {
  return {
    cells: table.fields.map((f: Field): Cell => {
      switch(f.options.type){
        case "Text":
          return "" as Text;
        case "Money":
          return 0 as Money;
        case "Integer":
          return 0 as Integer;
        case "Progress":
          return 0 as Progress;
        default:
          return "" as Text;
      }
    })
  };
};

const insertEntry = () => {
  insertEntryMode = true;
  newEntry = getNewEntry()
};

const saveEntry = () => {
  table.entries.push(newEntry);
  cancelEntry();
};

const cancelEntry = () => {
  insertEntryMode = false;
  newEntry = null as unknown as Entry;
};

</script>
<div class="flex flex-col items-center justify-center gap-3">
  {#await loadTable} 
    <p>Loading Table...</p>
  {:then _} 
    {@debug table, newEntry}
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
            {#each entry.cells as cell}
              <td class="border-2 border-gray-400 bg-white p-1 ">{cell}</td>
            {/each}
          </tr>
        {/each}
        {#if insertEntryMode}
          <tr>
            {#if newEntry === null}
              <td>error</td>
            {:else}
              {#each newEntry.cells as _, i }
                <td class="text-gray-500 border-2 border-gray-400 bg-white size-min">
                  <input bind:value={newEntry.cells[i]} class="border-none focus:outline-hidden outline-none size-full"/>
                </td>
              {/each}
            {/if}
          </tr>
        {/if}
      </tbody>
    </table> 
    {#if insertEntryMode}
      <div class="flex justify-center gap-3">
        <button onclick={saveEntry} class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition">Save</button>
        <button onclick={cancelEntry} class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition">Cancel</button>
      </div>
    {:else}
      <button onclick={insertEntry} class="text-center w-full mt-1 py-1 border-2 border-dashed border-gray-400 hover:bg-gray-400 transition">+ Add Row</button>
    {/if}
{err}
  {/await} 
</div>

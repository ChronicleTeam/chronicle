<script lang="ts">
import type { DataTable, Field, Entry,  Cell, Text, Money, Integer, Progress } from "$lib/types.d.js";
let { table_prop } = $props();

const table: DataTable = $state({
  table: table_prop,
  fields: [
    {
      name: "Project Name",
      options: {
          type: "Text",
          is_required: true
        }
    },
    {
      name: "Funding",
      options: {
        type: "Money",
        is_required: false
      }
    },
    {
      name: "Members",
      options: {
        type: "Integer",
        is_required: true
      }
    },
    {
      name: "Progress",
      options: {type: "Progress"}
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
});

let insertEntryMode = $state(false);
let newEntry = $state(null as unknown as Entry);

const getNewEntry = (t: DataTable): Entry => {
  return {
    cells: t.fields.map((f: Field): Cell => {
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
  newEntry = getNewEntry(table)
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
{@debug table, newEntry}
<div class="flex flex-col items-center justify-center">
  <table class=" border border-gray-400 bg-white text-black ">
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
    <button onclick={saveEntry} class="text-center mt-1 py-1 px-2 hover:py-2 transition-size duration-300 rounded bg-white">Save</button>
    <button onclick={cancelEntry} class="text-center mt-1 py-1 px-2 hover:py-2 transition-size duration-300 rounded bg-red-400 ">Cancel</button>
  {:else}
    <button onclick={insertEntry} class="text-center w-full mt-1 hover:mt-0 py-1 hover:py-2 transition-size duration-300 border-2 border-dashed border-gray-400">+ Add Row</button>
  {/if}

</div>

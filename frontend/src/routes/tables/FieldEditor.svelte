<script lang="ts">
import type { DataTable, Field, Entry,  Cell, Text, Money, Integer, Progress } from "$lib/types.d.js";
 

const fieldTypes = ["Text", "Money", "Integer", "Progress"];

let fields: Field[] = [
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
]
</script>

<div class="w-full">
  <!-- Top bar -->
  <h2 class="text-lg font-bold">Table</h2>
  <!-- Fields --> 
  <div class="flex items-center items-stretch gap-5 w-full overflow-scroll">
    {#each fields as field, i}
      <div class="bg-white p-3 rounded-lg">
        <input bind:value={fields[i].name} />
        <label for="typeSelect">Type: </label>
        <select id="typeSelect"  class="my-2" bind:value={fields[i].options.type}>
          {#each fieldTypes as fieldType}
            <option value={fieldType}>{fieldType}</option>
          {/each}
        </select>
      </div>
    {/each}
  </div>
</div>

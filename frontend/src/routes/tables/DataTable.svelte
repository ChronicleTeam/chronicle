<script lang="ts">
import type { DataTable, Field, Entry,  Cell, Text, Money, Integer, Progress, InputParameters } from "$lib/types.d.js";
import { FieldType } from "$lib/types.d.js"
import { API_URL } from "$lib/api.d.js";
import VariableInput from "$lib/components/VariableInput.svelte";
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
  EDIT: 2,
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
  table.entries.push(getNewEntry());
  editableEntry = table.entries.length-1;
};

const saveEntry = () => {
  fetch(`${API_URL}/tables/${table_prop.table_id}/entries`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(table.entries[table.entries.length-1].cells)
  }).then(cancelEntry).then(loadTable);
};

const cancelEntry = () => {
  if(entryMode === EntryMode.INSERT){
    table.entries.pop();
  }

  entryMode = EntryMode.DISPLAY;
  newEntry = null as unknown as Entry;
  editableEntry = -1;
};

let editableEntry = $state(-1);
const editEntry = (i : number) => {
  entryMode = EntryMode.EDIT;
  editableEntry = i;
}

const updateEntry = () => {
  fetch(`${API_URL}/tables/${table_prop.table_id}/entries/${table.entries[editableEntry].entry_id}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(table.entries[editableEntry].cells)
  }).then(cancelEntry).then(loadTable);
}

const cellToInputParams = (entryIdx: number, f: Field) => {
  switch(f.options.type){
    case FieldType.Integer:
    case FieldType.Money:
    case FieldType.Decimal:
    case FieldType.Progress:
      return {
        type: "number",
        bindGetter: () => table.entries[entryIdx].cells[f.field_id],
        bindSetter: (val: number) => {table.entries[entryIdx].cells[f.field_id] = val}
      } as InputParameters;
    case FieldType.DateTime:
      return {
        type: "date",
        bindGetter: () => (table.entries[entryIdx].cells[f.field_id] as Date).toISOString(),
        bindSetter: (val: string) => {table.entries[entryIdx].cells[f.field_id] = new Date(val)}
      } as InputParameters;
    case FieldType.Checkbox:
      return {
        type: "checkbox",
        bindGetter: () => table.entries[entryIdx].cells[f.field_id],
        bindSetter: (val: boolean) => {table.entries[entryIdx].cells[f.field_id] = val}
      } as InputParameters;
    case FieldType.Text:
    case FieldType.WebLink:
    case FieldType.Email:
    case FieldType.Enumeration:
    default:
      return {
        type: "text",
        bindGetter: () => table.entries[entryIdx].cells[f.field_id],
        bindSetter: (val: string) => {table.entries[entryIdx].cells[f.field_id] = val}
      } as InputParameters;
  }
}

$inspect(table, newEntry, entryMode, editableEntry);
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
        {#each table.entries as entry, i}
          <tr>
            {#each table.fields as field}
              <td class={["text-black border-2 border-gray-400 size-min", editableEntry === i && "bg-blue-200", editableEntry !== i && "bg-white"]} onclick={() => {if(entryMode === EntryMode.DISPLAY) editEntry(i)}}>
                <VariableInput disabled={i !== editableEntry} innerClass={["border-none focus:outline-hidden outline-none size-full disabled:pointer-events-none", editableEntry === i && "bg-blue-200", editableEntry !== i && "bg-white"]} params={cellToInputParams(i, field)}/>
              </td>
            {/each}
          </tr>
        {/each}
      <!--
        {#if entryMode === EntryMode.INSERT}
          <tr>
            {#if newEntry === null}
              <td>error</td>
            {:else}
              {#each Object.keys(newEntry.cells) as key }
                <td class="text-gray-500 border-2 border-gray-400 bg-white size-min">
                  <VariableInput disabled={i !== editableEntry} innerClass="border-none focus:outline-hidden outline-none size-full disabled:pointer-events-none" params={cellToInputParams(i, field)}/>
                  <input bind:value={newEntry.cells[key]} class="border-none focus:outline-hidden outline-none size-full"/>
                </td>
              {/each}
            {/if}
          </tr>
        {/if} -->
      </tbody>
    </table> 
    {#if entryMode === EntryMode.INSERT || entryMode === EntryMode.EDIT}
      <div class="flex justify-center gap-3">
        <button onclick={entryMode === EntryMode.INSERT ? saveEntry : updateEntry} class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition">Save</button>
        <button onclick={cancelEntry} class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition">Cancel</button>
      </div>
    {:else if entryMode === EntryMode.DISPLAY && table.fields.length > 0}
      <button onclick={insertEntry} class="text-center w-full mt-1 py-1 border-2 border-dashed border-gray-400 hover:bg-gray-400 transition">+ Add Row</button>
    {/if}
{err}
</div>

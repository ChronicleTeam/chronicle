<script lang="ts">
  import type {
    DataTable,
    Field,
    Entry,
    Cell,
    Text,
    Integer,
    Decimal,
    Money,
    Progress,
    DateTime,
    Interval,
    Weblink,
    Email,
    Checkbox,
    Enumeration,
    InputParameters,
    EnumerationKind,
  } from "$lib/types.d.js";
  import { FieldType, parseJSONTable } from "$lib/types.d.js";
  import { API_URL } from "$lib/api.d.js";
  import VariableInput from "$lib/components/VariableInput.svelte";
  let { table_prop } = $props();

  let err = $state();

  const loadTable = () => {
    fetch(`${API_URL}/tables/${table_prop.table_id}/data`)
      .then((response) => response.json())
      .then((json) => {
        table = parseJSONTable(json);
      });
  };

  const EntryMode = {
    DISPLAY: 0,
    INSERT: 1,
    EDIT: 2,
  };

  let table = $state({
    table: table_prop,
    fields: [],
    entries: [],
  } as DataTable);

  loadTable();

  let entryMode = $state(EntryMode.DISPLAY);

  const getNewEntry = (): Entry => {
    return {
      entry_id: -1,
      cells: Object.fromEntries(
        table.fields.map((f: Field): [string, Cell] => {
          switch (f.field_kind.type) {
            case FieldType.Text:
              return [f.field_id.toString(), "" as Text];
            case FieldType.Integer:
              return [f.field_id.toString(), 0 as Integer];
            case FieldType.Decimal:
              return [f.field_id.toString(), 0 as Decimal];
            case FieldType.Money:
              return [f.field_id.toString(), 0 as Money];
            case FieldType.Progress:
              return [f.field_id.toString(), 0 as Progress];
            case FieldType.DateTime:
              return [f.field_id.toString(), new Date() as DateTime];
            case FieldType.Interval:
              return [f.field_id.toString(), null as Interval];
            case FieldType.WebLink:
              return [f.field_id.toString(), "" as Weblink];
            case FieldType.Email:
              return [f.field_id.toString(), "" as Email];
            case FieldType.Checkbox:
              return [f.field_id.toString(), false as Checkbox];
            case FieldType.Enumeration:
              return [f.field_id.toString(), 0 as Enumeration];
            default:
              return [f.field_id.toString(), "" as Text];
          }
        }),
      ),
    };
  };

  const insertEntry = () => {
    entryMode = EntryMode.INSERT;
    table.entries.push(getNewEntry());
    editableEntry = table.entries.length - 1;
  };

  const saveEntry = () => {
    fetch(`${API_URL}/tables/${table_prop.table_id}/entries`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(table.entries[table.entries.length - 1].cells),
    }).then(async (response) => {
      if (response.status === 200) {
        cancelEntry();
        loadTable();
      } else if (response.status === 422) {
        fieldErrors = await response.json();
      }
    });
  };

  const cancelEntry = () => {
    if (entryMode === EntryMode.INSERT) {
      table.entries.pop();
    }

    entryMode = EntryMode.DISPLAY;
    editableEntry = -1;
    fieldErrors = {};
  };

  let editableEntry = $state(-1);
  let deleteConfirmation = $state(false);
  const editEntry = (i: number) => {
    entryMode = EntryMode.EDIT;
    editableEntry = i;
    deleteConfirmation = false;
  };
  let fieldErrors = $state({} as { [key: number]: string });
  const updateEntry = () => {
    fetch(
      `${API_URL}/tables/${table_prop.table_id}/entries/${table.entries[editableEntry].entry_id}`,
      {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(table.entries[editableEntry].cells),
      },
    ).then(async (response) => {
      if (response.status === 200) {
        cancelEntry();
        loadTable();
      } else if (response.status === 422) {
        fieldErrors = await response.json();
      }
    });
  };

  const cellToInputParams = (entryIdx: number, f: Field) => {
    switch (f.field_kind.type) {
      case FieldType.Integer:
      case FieldType.Money:
      case FieldType.Decimal:
      case FieldType.Progress:
        return {
          type: "number",
          bindGetter: () => table.entries[entryIdx].cells[f.field_id],
          bindSetter: (val: number) => {
            table.entries[entryIdx].cells[f.field_id] = val;
          },
        } as InputParameters;
      case FieldType.DateTime:
        return {
          type: "datetime-local",
          bindGetter: () =>
            ((table.entries[entryIdx].cells[f.field_id] as Date) ?? new Date())
              .toISOString()
              .substring(0, 19),
          bindSetter: (val: string) => {
            table.entries[entryIdx].cells[f.field_id] = new Date(val);
          },
        } as InputParameters;
      case FieldType.Checkbox:
        return {
          type: "checkbox",
          bindGetter: () => table.entries[entryIdx].cells[f.field_id],
          bindSetter: (val: boolean) => {
            table.entries[entryIdx].cells[f.field_id] = val;
          },
        } as InputParameters;
      case FieldType.Enumeration:
        return {
          type: "select",
          selectOptions: Object.values(f.field_kind.values),
          bindGetter: () =>
            (f.field_kind as EnumerationKind).values[
              table.entries[entryIdx].cells[f.field_id] as number
            ],
          bindSetter: (val: string) => {
            table.entries[entryIdx].cells[f.field_id] = parseInt(
              (Object.entries((f.field_kind as EnumerationKind).values).find(
                (e) => e[1] === val,
              ) ?? ["0"])[0],
            );
          },
        } as InputParameters;
      case FieldType.Text:
      case FieldType.WebLink:
      case FieldType.Email:
      default:
        return {
          type: "text",
          bindGetter: () => table.entries[entryIdx].cells[f.field_id],
          bindSetter: (val: string) => {
            table.entries[entryIdx].cells[f.field_id] = val;
          },
        } as InputParameters;
    }
  };

  const deleteEntry = () => {
    if (editableEntry === -1) return;

    fetch(
      `${API_URL}/tables/${table_prop.table_id}/entries/${table.entries[editableEntry].entry_id}`,
      {
        method: "DELETE",
      },
    )
      .then(cancelEntry)
      .then(loadTable);
  };

  $inspect(table, entryMode, editableEntry);
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
            <td
              class={[
                "relative text-black border-2 border-gray-400 size-min",
                editableEntry === i && "bg-blue-200",
                editableEntry !== i && "bg-white",
              ]}
              onclick={() => {
                if (entryMode === EntryMode.DISPLAY) editEntry(i);
              }}
            >
              {#if editableEntry === i && fieldErrors[field.field_id] !== undefined}
                <div
                  class="absolute bottom-full inset-x-0 flex flex-col items-center"
                >
                  <div
                    class="bg-gray-100 text-center p-3 mx-1 mt-1 rounded-lg text-red-500 text-sm"
                  >
                    Error: {fieldErrors[field.field_id]}
                  </div>
                  <svg width="20" height="10">
                    <polygon points="0,0 20,0 10,10" class="fill-gray-100" />
                  </svg>
                </div>
              {/if}
              <VariableInput
                disabled={i !== editableEntry}
                innerClass={[
                  "border-none focus:outline-hidden outline-none size-full disabled:pointer-events-none",
                  editableEntry === i && "bg-blue-200",
                  editableEntry !== i && "bg-white",
                ]}
                params={cellToInputParams(i, field)}
              />
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
  {#if entryMode === EntryMode.INSERT || entryMode === EntryMode.EDIT}
    <div class="flex justify-center gap-3">
      <button
        onclick={entryMode === EntryMode.INSERT ? saveEntry : updateEntry}
        class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
        >Save</button
      >
      <button
        onclick={cancelEntry}
        class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition"
        >Cancel</button
      >
      {#if entryMode === EntryMode.EDIT}
        {#if deleteConfirmation}
          <button
            onclick={deleteEntry}
            class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition"
            >Confirm delete</button
          >
        {:else}
          <button
            onclick={() => {
              deleteConfirmation = true;
            }}
            class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
            >Delete Entry</button
          >
        {/if}
      {/if}
    </div>
  {:else if entryMode === EntryMode.DISPLAY && table.fields.length > 0}
    <button
      onclick={insertEntry}
      class="text-center w-full mt-1 py-1 border-2 border-dashed border-gray-400 hover:bg-gray-400 transition"
      >+ Add Row</button
    >
  {/if}
  {err}
</div>

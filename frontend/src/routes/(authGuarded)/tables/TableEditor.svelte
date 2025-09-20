<script lang="ts">
  import type {
    Table,
    TableData,
    Field,
    Entry,
    Cell,
    Text,
    Integer,
    Decimal,
    Money,
    Progress,
    DateTime,
    Weblink,
    Checkbox,
    Enumeration,
    InputParameters,
    TextKind,
    EnumerationKind,
    IntegerKind,
    MoneyKind,
    DecimalKind,
  } from "$lib/types";
  import { FieldType } from "$lib/types";
  import {
    getTableData,
    postEntries,
    patchEntry,
    deleteEntry,
    type APIError,
  } from "$lib/api";
  import VariableInput from "$lib/components/VariableInput.svelte";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import TableEditor from "./TableEditor.svelte";
  import FieldEditor from "./FieldEditor.svelte";
  import { onMount } from "svelte";
  import { TableMode, type ModeState, type TableChild } from "./types";
  let { table_prop, entry_id }: { table_prop: Table; entry_id?: number } =
    $props();

  //
  // State
  //

  // the TableData object being displayed
  let table = $state({
    table: {
      table_id: 1,
      user_id: 1,
      name: "Users",
      description: "",
    },
    fields: [
      {
        table_id: 1,
        user_id: 1,
        field_id: 1,
        name: "Name",

        ordering: 1,
        field_kind: {
          type: FieldType.Text,
          is_required: false,
        },
      },
      {
        table_id: 1,
        user_id: 1,
        field_id: 2,
        name: "Last Name",

        ordering: 1,
        field_kind: {
          type: FieldType.Text,
          is_required: false,
        },
      },
    ],
    entries: [
      {
        entry_id: 1,
        cells: {
          "1": "Bob",
          "2": "Smith",
        },
      },
      {
        entry_id: 2,
        cells: {
          "1": "John",
          "2": "Smith",
        },
      },
    ],
    children: [
      {
        table: {
          table_id: 2,
          user_id: 1,
          parent_id: 1,
          name: "Friends",
          description: "",
        },
        fields: [
          {
            table_id: 2,
            user_id: 1,
            field_id: 1,
            name: "Name",

            ordering: 1,
            field_kind: {
              type: FieldType.Text,
              is_required: false,
            },
          },
          {
            table_id: 2,
            user_id: 1,
            field_id: 2,
            name: "Last Name",

            ordering: 1,
            field_kind: {
              type: FieldType.Text,
              is_required: false,
            },
          },
        ],
        entries: [
          {
            entry_id: 1,
            cells: {
              "1": "Bob",
              "2": "Smith",
            },
          },
          {
            entry_id: 2,
            cells: {
              "1": "John",
              "2": "Smith",
            },
          },
        ],
        children: [],
      },
    ],
  } as TableData);

  // mode-dependent variables
  let modeState: ModeState = $state({ mode: TableMode.DISPLAY });
  const modeDisplay = () => {
    modeState = { mode: TableMode.DISPLAY };
  };
  const modeInsert = (entry_idx: number) => {
    modeState = { mode: TableMode.INSERT, entry_idxes: [entry_idx] };
  };
  const modeEdit = (entry_idx: number) => {
    modeState = { mode: TableMode.EDIT, entry_idx };
  };
  const modeChild = (child: TableChild) => {
    modeState = { mode: TableMode.CHILD, child };
  };
  const modeEditChild = (child: TableChild) => {
    modeState = { mode: TableMode.EDIT_CHILD, child };
  };

  /**
   * Cancel addition of entries
   */
  const cancelEntries = () => {
    if (modeState.mode === TableMode.INSERT) {
      modeState.entry_idxes.forEach(() => table.entries.pop());
    }

    modeDisplay();
    errors.fields = {};
  };

  /**
   * Insert a placeholder entry to be added
   */
  const insertEntry = () => {
    table.entries.push(getNewEntry());

    if (!(modeState.mode === TableMode.INSERT)) {
      modeInsert(table.entries.length - 1);
    } else {
      modeState.entry_idxes.push(table.entries.length - 1);
    }
  };

  /**
   * Edit a particular entry
   * @param {number} i - The index of the entry in table.entries
   */
  const editEntry = modeEdit;

  let errors: {
    fields:
      | {
          [key: string]: string;
        }
      | string;
  } = $state({
    fields: {},
  });

  //
  // Helper methods
  //

  /**
   * Generate a new blank Entry object
   * @returns {Entry}
   */
  const getNewEntry = (): Entry => {
    let i = -1;
    while (table.entries.some((e) => e.entry_id === i)) {
      i--;
    }
    return {
      parent_id: entry_id ?? null,
      entry_id: i,
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
              return [f.field_id.toString(), "0.00" as Money];
            case FieldType.Progress:
              return [f.field_id.toString(), 0 as Progress];
            case FieldType.DateTime:
              return [f.field_id.toString(), new Date() as DateTime];
            case FieldType.WebLink:
              return [f.field_id.toString(), "" as Weblink];
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

  /**
   * Generate InputParams for a specific Cell
   * @param {number} entryIdx - The index of the Entry of the Cell in table.entries
   * @param {Field} f - The Field of the Cell
   * @returns {InputParameters}
   */
  const cellToInputParams = (entryIdx: number, f: Field): InputParameters => {
    switch (f.field_kind.type) {
      case FieldType.Integer:
      case FieldType.Decimal:
        return {
          type: "number",
          bindGetter: () => table.entries[entryIdx].cells[f.field_id],
          bindSetter: (val: number) => {
            table.entries[entryIdx].cells[f.field_id] = val;
          },
          min: (f.field_kind as IntegerKind | MoneyKind | DecimalKind)
            .range_start,
          max: (f.field_kind as IntegerKind | MoneyKind | DecimalKind)
            .range_end,
          step:
            f.field_kind.type === FieldType.Integer
              ? 1
              : Math.pow(
                  10,
                  -((f.field_kind as DecimalKind).number_scale ?? 10),
                ),
        } as InputParameters;
      case FieldType.Money:
        return {
          type: "number",
          bindGetter: () =>
            parseFloat(table.entries[entryIdx].cells[f.field_id] as string),
          bindSetter: (val: number) => {
            table.entries[entryIdx].cells[f.field_id] = val.toFixed(2);
          },
          min:
            (f.field_kind as MoneyKind).range_start != null
              ? parseFloat((f.field_kind as MoneyKind).range_start as string)
              : undefined,
          max:
            (f.field_kind as MoneyKind).range_end != null
              ? parseFloat((f.field_kind as MoneyKind).range_end as string)
              : undefined,
          step: 0.01,
        } as InputParameters;
      case FieldType.Progress:
        return {
          type: "number",
          bindGetter: () => table.entries[entryIdx].cells[f.field_id],
          bindSetter: (val: number) => {
            table.entries[entryIdx].cells[f.field_id] = val;
          },
          min: 0,
          max: f.field_kind.total_steps,
          step: 1,
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
          min: f.field_kind.range_start,
          max: f.field_kind.range_end,
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
      case FieldType.WebLink:
        return {
          type: "url",
          bindGetter: () => table.entries[entryIdx].cells[f.field_id],
          bindSetter: (val: string) => {
            table.entries[entryIdx].cells[f.field_id] = val;
          },
        } as InputParameters;
      case FieldType.Text:
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

  //
  // API Calls
  //

  const loadTable = () => {
    getTableData(table_prop).then((response: TableData) => {
      response.fields.sort((f, g) => f.ordering - g.ordering);

      if (entry_id) {
        response.entries = response.entries.filter(
          (e) => e.parent_id === entry_id,
        );
      }
      table = response;
    });
  };

  const createEntries = () => {
    if (modeState.mode === TableMode.INSERT) {
      postEntries(
        table.table,
        table.entries.filter((e, idx) =>
          modeState.mode === TableMode.INSERT
            ? modeState.entry_idxes.some((i: number) => idx === i)
            : false,
        ),
      )
        .then(() => {
          cancelEntries();
          loadTable();
        })
        .catch((e: APIError) => {
          if (e.status === 422) {
            errors.fields = e.body;
          }
        });
    }
  };

  const updateEntry = () => {
    if (modeState.mode === TableMode.EDIT) {
      patchEntry(table.table, table.entries[modeState.entry_idx])
        .then(() => {
          cancelEntries();
          loadTable();
        })
        .catch((e: APIError) => {
          if (e.status === 422) {
            errors.fields = e.body;
          }
        });
    }
  };

  const removeEntry = () => {
    if (modeState.mode === TableMode.EDIT) {
      deleteEntry(table.table, table.entries[modeState.entry_idx])
        .then(cancelEntries)
        .then(loadTable);
    }
  };

  //
  // Startup
  //
  onMount(() => {
    loadTable();
  });
</script>

{#if modeState.mode === TableMode.CHILD}
  <!-- Display child table -->
  <div class="flex items-center gap-2 mb-2">
    <button
      class="btn"
      onclick={() => {
        modeDisplay();
        loadTable();
      }}>Back to <span class="font-bold">{table.table.name}</span></button
    >
    <h2 class="text-lg font-bold">{modeState.child.table_data.table.name}</h2>
  </div>
  <TableEditor
    table_prop={modeState.child.table_data.table}
    entry_id={modeState.child.entry_id}
  />
{:else if modeState.mode === TableMode.EDIT_CHILD}
  <FieldEditor
    table_prop={modeState.child.table_data.table}
    on_save={() => {
      modeDisplay();
    }}
    delete_table={() => {}}
  />
{:else}
  <!-- Display main table -->
  <div class="flex flex-col items-center justify-center gap-3">
    <!-- Main table -->
    <div
      class="overflow-x-auto rounded-lg border border-base-content/5 border-base-100"
    >
      <table class="table text-base-content w-full">
        <!-- Headers -->
        <thead>
          <tr>
            {#each table.fields as field}
              <th>
                <!-- Floating error bubble -->
                <div
                  class={{
                    "tooltip tooltip-error tooltip-right ": true,
                    "tooltip-open":
                      modeState.mode === TableMode.INSERT &&
                      typeof errors.fields === "object" &&
                      errors.fields[field.field_id.toString()] !== undefined,
                  }}
                  data-tip={modeState.mode === TableMode.INSERT &&
                  typeof errors.fields === "object" &&
                  errors.fields[field.field_id.toString()] !== undefined
                    ? `Error: ${errors.fields[field.field_id.toString()]}`
                    : undefined}
                >
                  {field.name}
                </div>
              </th>
            {/each}
            {#each table.children as child}
              <th
                >{child.table.name}
                <button
                  class="btn"
                  onclick={() => {
                    modeEditChild({ table_data: child, entry_id: -1 });
                  }}
                >
                  Edit</button
                ></th
              >
            {/each}
          </tr>
        </thead>

        <!-- Cells -->
        <tbody>
          {#each table.entries.filter( (e) => (entry_id != null ? e.parent_id === entry_id : true), ) as entry, i}
            <tr>
              <!-- Regular Cells -->
              {#each table.fields as field, j}
                <td
                  class={[
                    (modeState.mode === TableMode.INSERT &&
                      modeState.entry_idxes.includes(i)) ||
                    (modeState.mode === TableMode.EDIT &&
                      modeState.entry_idx === i)
                      ? "bg-info/25"
                      : "bg-base-100",
                  ]}
                  ondblclick={() => {
                    if (modeState.mode === TableMode.DISPLAY) editEntry(i);
                  }}
                >
                  <!-- Floating error bubble -->
                  <div
                    class={{
                      "tooltip tooltip-error": true,
                      "tooltip-open":
                        modeState.mode === TableMode.EDIT &&
                        modeState.entry_idx === i &&
                        typeof errors.fields === "object" &&
                        errors.fields[field.field_id.toString()] !== undefined,
                    }}
                    data-tip={modeState.mode === TableMode.EDIT &&
                    modeState.entry_idx === i &&
                    typeof errors.fields === "object" &&
                    errors.fields[field.field_id.toString()] !== undefined
                      ? `Error: ${errors.fields[field.field_id.toString()]}`
                      : undefined}
                  >
                    <!-- Table cell -->
                    <VariableInput
                      id={`input-${i}-${j}`}
                      disabled={!(
                        (modeState.mode === TableMode.INSERT &&
                          modeState.entry_idxes.includes(i)) ||
                        (modeState.mode === TableMode.EDIT &&
                          modeState.entry_idx === i)
                      )}
                      class="border-none bg-transparent focus:outline-hidden outline-hidden size-full disabled:pointer-events-none"
                      params={cellToInputParams(i, field)}
                      onkeydown={(k) => {
                        if (k.key === "Enter") {
                          if (i === table.entries.length - 1) {
                            insertEntry();
                          } else {
                            document
                              .getElementById(`input-${i + 1}-${0}`)
                              ?.focus();
                          }
                        }
                      }}
                    />
                  </div>
                </td>
              {/each}

              <!-- Child table Cells -->
              {#each table.children as child}
                <td
                  class={[
                    (modeState.mode === TableMode.INSERT &&
                      modeState.entry_idxes.includes(i)) ||
                    (modeState.mode === TableMode.EDIT &&
                      modeState.entry_idx === i)
                      ? "bg-info/25"
                      : "bg-base-100",
                  ]}
                  onclick={() => {
                    if (modeState.mode === TableMode.EDIT) {
                      modeChild({
                        table_data: child,
                        entry_id: entry.entry_id,
                      });
                    }
                  }}
                  ondblclick={() => {
                    if (modeState.mode === TableMode.DISPLAY) {
                      modeChild({
                        table_data: child,
                        entry_id: entry.entry_id,
                      });
                    }
                  }}
                >
                  <p>
                    {child.entries.filter((e) => e.parent_id === entry.entry_id)
                      .length} entries
                  </p>
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    {#if typeof errors.fields === "string"}
      <p class="text-error">
        Error: {errors.fields}
      </p>
    {/if}
    <!-- Add row button -->
    {#if (modeState.mode === TableMode.DISPLAY || modeState.mode === TableMode.INSERT) && table.fields.length > 0}
      <button onclick={insertEntry} class="btn btn-dash btn-block border-2"
        >+ Add Row</button
      >
    {/if}
    <!-- Button cluster to confirm/cancel editing/creating entries -->
    {#if modeState.mode === TableMode.INSERT || modeState.mode === TableMode.EDIT}
      <div class="flex justify-center gap-3">
        <button
          onclick={modeState.mode === TableMode.INSERT
            ? createEntries
            : updateEntry}
          class="btn">Save</button
        >
        {#if modeState.mode === TableMode.EDIT}
          <ConfirmButton
            initText="Delete Entry"
            confirmText="Confirm Delete"
            onconfirm={removeEntry}
          />
        {/if}
        <button onclick={cancelEntries} class="btn btn-error">Cancel</button>
      </div>
    {/if}
  </div>
{/if}

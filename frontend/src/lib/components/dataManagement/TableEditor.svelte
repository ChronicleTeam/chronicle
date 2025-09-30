<script lang="ts">
  import type {
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
    EnumerationKind,
    IntegerKind,
    MoneyKind,
    DecimalKind,
    TableData,
  } from "$lib/types";
  import { FieldType } from "$lib/types";
  import {
    postEntries,
    patchEntry,
    deleteEntry,
    postExportTable,
    type APIError,
  } from "$lib/api";
  import VariableInput from "$lib/components/VariableInput.svelte";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import { TableMode, type ModeState } from "./types";
  import { goto, refreshAll } from "$app/navigation";

  //
  // State
  //

  // the TableData object being displayed
  let {
    entryId: entryIdProp = null,
    table: tableProp,
  }: { entryId?: string | null; table: TableData } = $props();
  let entryId = $derived(entryIdProp);
  let table = $state(tableProp);
  $effect(() => {
    table = tableProp;
  });

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
    table: {
      export: string;
    };
  } = $state({
    fields: {},
    table: {
      export: "",
    },
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
      parent_id: entryId ? parseInt(entryId) : null,
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
          refreshAll();
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
          refreshAll();
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
        .then(() => refreshAll());
    }
  };

  const FileTypes = {
    csv: "text/csv",
    excel: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  };

  const FileExtensions = {
    csv: ".csv",
    excel: ".xlsx",
  };
  const exportTable = (type: "csv" | "excel") => {
    postExportTable(table.table, type)
      .then((r) => {
        let exportedFile = new File(
          [r],
          table.table.name.replaceAll(" ", "_") + FileExtensions[type],
          {
            type: FileTypes[type],
          },
        );
        open(URL.createObjectURL(exportedFile));
      })
      .catch((e) => {
        errors.table.export = e.body.toString();
      });
  };
</script>

<!-- Display main table -->
<div class=" flex flex-col items-center gap-3">
  <!-- Top bar -->
  <div class="flex justify-between items-center gap-2">
    <div></div>
    <h2 class="text-lg font-bold">{table.table.name}</h2>
    <div>
      {#if entryId == null}
        <button
          onclick={() => {
            goto(`/tables/${table.table.table_id}/edit`);
          }}
          class="btn">Edit</button
        >
      {/if}
      <!-- Export buttons -->
      <details class="dropdown">
        <summary class="btn">Export</summary>
        <ul class="dropdown-content menu bg-base-100 rounded-md m-1 w-64">
          <li>
            <button onclick={() => exportTable("csv")}>Export as CSV</button>
          </li>
          <li>
            <button onclick={() => exportTable("excel")}
              >Export as Excel Spreadsheet</button
            >
          </li>
        </ul>
      </details>
    </div>
  </div>

  {#if errors.table.export}
    <p class="text-error">Could not export table.</p>
  {/if}
  <h3 class="text-lg mb-2">{table.table.description}</h3>

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
                  goto(
                    `/tables/${table.table.table_id}/subtables/${child.table.table_id}/edit`,
                  );
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
        {#each table.entries as entry, i}
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
                    goto(
                      `/tables/${table.table.table_id}/subtables/${child.table.table_id}/${entry.entry_id}`,
                    );
                  }
                }}
                ondblclick={() => {
                  if (modeState.mode === TableMode.DISPLAY) {
                    goto(
                      `/tables/${table.table.table_id}/subtables/${child.table.table_id}/${entry.entry_id}`,
                    );
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

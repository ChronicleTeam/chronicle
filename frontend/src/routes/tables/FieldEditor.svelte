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
    type TextKind,
    type MoneyKind,
    type IntegerKind,
    type ProgressKind,
    FieldType,
    type DecimalKind,
    type DateTimeKind,
    type IntervalKind,
    type WebLinkKind,
    type EmailKind,
    type EnumerationKind,
    type ImageKind,
    type FileKind,
    type FieldKind,
    type InputParameters,
    type Table,
  } from "$lib/types.d.js";
  import VariableInput from "$lib/components/VariableInput.svelte";
  import {
    putTable,
    getFields,
    postField,
    putField,
    deleteField,
    type APIError,
  } from "$lib/api.js";

  let { table_prop, on_save, delete_table } = $props();

  let originalTable: DataTable = $state({
    table: table_prop,
    fields: [],
    entries: [],
  });

  let table = $state($state.snapshot(originalTable));

  const loadFields = () => {
    getFields(table_prop).then((fields) => {
      originalTable.fields = fields;
      table = $state.snapshot(originalTable);
      optionalCheckboxStates = optionInputList.map((val) =>
        val.map((v) => !v.optional),
      );
      table.fields.forEach((f) => {
        fieldErrors[f.field_id] = "";
      });
    });
  };
  loadFields();
  const fieldTypes = Object.values(FieldType);

  type OptionInputParameters = InputParameters & {
    optional: boolean;
    name: string;
  };

  const getTypeOptionInput = (i: number): OptionInputParameters => {
    return {
      name: "type",
      label: "Type",
      type: "select",
      optional: false,
      selectOptions: fieldTypes,
      bindGetter: () => {
        return table.fields[i].field_kind.type;
      },
      bindSetter: (val: FieldType) => {
        // swap out field option if type change
        if (val != table.fields[i].field_kind.type) {
          switch (val) {
            case FieldType.Text:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Integer:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Decimal:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,

                scientific_notation: true,
              };
              break;
            case FieldType.Money:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Progress:
              table.fields[i].field_kind = {
                type: val,
                total_steps: 100,
              };
              break;
            case FieldType.DateTime:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,

                date_time_format: "YYYY-MM-DD",
              };
              break;
            case FieldType.Interval:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.WebLink:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Email:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Checkbox:
              table.fields[i].field_kind = {
                type: val,
              };
              break;
            case FieldType.Enumeration:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
                values: {} as { [key: number]: string },
                default_value: 0,
              };
              break;
            case FieldType.Image:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.File:
              table.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
          }
          optionalCheckboxStates[i] = optionInputList[i].map(
            (v) => !v.optional,
          );
        }
      },
    };
  };

  type RequirableKind =
    | TextKind
    | IntegerKind
    | DecimalKind
    | MoneyKind
    | DateTimeKind
    | IntervalKind
    | WebLinkKind
    | EmailKind
    | EnumerationKind
    | ImageKind
    | FileKind;

  const getRequiredOptionInput = (i: number): OptionInputParameters => {
    return {
      name: "is_required",
      label: "Is Required",
      type: "checkbox",
      optional: false,
      bindGetter: () => {
        return (table.fields[i].field_kind as RequirableKind).is_required;
      },
      bindSetter: (val: boolean) => {
        (table.fields[i].field_kind as RequirableKind).is_required = val;
      },
    };
  };

  const optionInputList = $derived(
    table.fields.map((f: Field, i: number): OptionInputParameters[] => {
      switch (f.field_kind.type) {
        case FieldType.Text:
          return [getTypeOptionInput(i), getRequiredOptionInput(i)];
        case FieldType.Integer:
          return [
            getTypeOptionInput(i),
            getRequiredOptionInput(i),
            {
              name: "range_start",
              label: "Range start",
              type: "number",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as IntegerKind).range_start ?? 0
                );
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as IntegerKind).range_start = val;
              },
            },
            {
              name: "range_end",
              label: "Range end",
              type: "number",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as IntegerKind).range_end ?? 100
                );
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as IntegerKind).range_end = val;
              },
            },
          ];
        case FieldType.Decimal:
          return [
            getTypeOptionInput(i),
            getRequiredOptionInput(i),
            {
              name: "range_start",
              label: "Range start",
              type: "number",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as DecimalKind).range_start ?? 0
                );
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as DecimalKind).range_start = val;
              },
            },
            {
              name: "range_end",
              label: "Range end",
              type: "number",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as DecimalKind).range_end ?? 0
                );
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as DecimalKind).range_end = val;
              },
            },
            {
              name: "scientific_notation",
              label: "Scientific notation",
              type: "checkbox",
              optional: false,
              bindGetter: () => {
                return (table.fields[i].field_kind as DecimalKind)
                  .scientific_notation;
              },
              bindSetter: (val: boolean) => {
                (
                  table.fields[i].field_kind as DecimalKind
                ).scientific_notation = val;
              },
            },
            {
              name: "number_precision",
              label: "Number Precision",
              type: "number",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as DecimalKind)
                    .number_precision ?? 0
                );
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as DecimalKind).number_precision =
                  val;
              },
            },
            {
              name: "number_scale",
              label: "Number Scale",
              type: "number",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as DecimalKind).number_scale ?? 0
                );
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as DecimalKind).number_scale = val;
              },
            },
          ];
        case FieldType.Money:
          return [
            getTypeOptionInput(i),
            getRequiredOptionInput(i),
            {
              name: "range_start",
              label: "Range start",
              type: "number",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as MoneyKind).range_start ?? 0
                );
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as MoneyKind).range_start = val;
              },
            },
            {
              name: "range_end",
              label: "Range end",
              type: "number",
              optional: true,
              bindGetter: () => {
                return (table.fields[i].field_kind as MoneyKind).range_end ?? 0;
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as MoneyKind).range_end = val;
              },
            },
          ];
        case FieldType.Progress:
          return [
            getTypeOptionInput(i),
            {
              name: "total_steps",
              label: "Total steps",
              type: "number",
              optional: false,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as ProgressKind).total_steps ?? 0
                );
              },
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as ProgressKind).total_steps = val;
              },
            },
          ];
        case FieldType.DateTime:
          return [
            getTypeOptionInput(i),
            getRequiredOptionInput(i),
            {
              name: "range_start",
              label: "Range start",
              type: "datetime-local",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as DateTimeKind).range_start
                    ?.toISOString()
                    .substring(0, 19) ??
                  new Date().toISOString().substring(0, 19)
                );
              },
              bindSetter: (val: string) => {
                (table.fields[i].field_kind as DateTimeKind).range_start =
                  new Date(val);
              },
            },
            {
              name: "range_end",
              label: "Range end",
              type: "datetime-local",
              optional: true,
              bindGetter: () => {
                return (
                  (table.fields[i].field_kind as DateTimeKind).range_end
                    ?.toISOString()
                    .substring(0, 19) ??
                  new Date().toISOString().substring(0, 19)
                );
              },
              bindSetter: (val: string) => {
                (table.fields[i].field_kind as DateTimeKind).range_end =
                  new Date(val);
              },
            },
            {
              name: "date_time_format",
              label: "DateTime format",
              type: "text",
              optional: false,
              bindGetter: () => {
                return (table.fields[i].field_kind as DateTimeKind)
                  .date_time_format;
              },
              bindSetter: (val: string) => {
                (table.fields[i].field_kind as DateTimeKind).date_time_format =
                  val;
              },
            },
          ];
        case FieldType.Interval:
          return [getTypeOptionInput(i), getRequiredOptionInput(i)];
        case FieldType.WebLink:
          return [getTypeOptionInput(i), getRequiredOptionInput(i)];
        case FieldType.Email:
          return [getTypeOptionInput(i), getRequiredOptionInput(i)];
        case FieldType.Checkbox:
          return [getTypeOptionInput(i)];
        case FieldType.Enumeration:
          // TODO: Add map input somehow
          return [
            getTypeOptionInput(i),
            getRequiredOptionInput(i),
            {
              name: "values",
              label: "Values",
              type: "textarea",
              optional: false,
              bindGetter: () => {
                return Object.entries(
                  (table.fields[i].field_kind as EnumerationKind).values,
                )
                  .map((entry) => entry[0].toString() + ":" + entry[1])
                  .join("\n");
              },
              bindSetter: (val: string) => {
                (table.fields[i].field_kind as EnumerationKind).values =
                  Object.fromEntries(
                    val
                      .split("\n")
                      .map((line) => {
                        let entry = line.split(":", 2);
                        return entry.length < 2 ? [entry[0], ""] : entry;
                      }) // split
                      .map((entry: string[]) => [parseInt(entry[0]), entry[1]]) // parse
                      .map(
                        (entry, i, arr) => {
                          if (isNaN(entry[0] as number)) {
                            let i = 0;
                            while (arr.some((e) => e[0] === i)) {
                              i++;
                            }
                            return [i, entry[1]];
                          } else {
                            return entry;
                          }
                        }, // catch NaN
                      ) as [number, string][],
                  );
              },
            },
            {
              name: "default_value",
              label: "Default value",
              type: "number",
              optional: false,
              bindGetter: () =>
                (table.fields[i].field_kind as EnumerationKind).default_value,
              bindSetter: (val: number) => {
                (table.fields[i].field_kind as EnumerationKind).default_value =
                  val;
              },
            },
          ];
        case FieldType.Image:
          return [getTypeOptionInput(i), getRequiredOptionInput(i)];
        case FieldType.File:
          return [getTypeOptionInput(i), getRequiredOptionInput(i)];
        default:
          return [];
      }
    }),
  );

  const addField = (i: number): void => {
    let j = 1;
    let newFieldName = "New Field " + j;
    while (table.fields.some((f: Field) => f.name === newFieldName)) {
      newFieldName = "New Field " + ++j;
    }

    let id = -1;
    while (table.fields.some((f) => f.field_id === id)) {
      id--;
    }
    let newField: Field = {
      table_id: table.table.table_id,
      user_id: -1,
      field_id: id, // temporary id, will be replaced when created

      name: newFieldName,
      field_kind: {
        type: FieldType.Text,
        is_required: true,
      },
    };

    table.fields.splice(i + 1, 0, newField);
    optionalCheckboxStates.splice(
      i + 1,
      0,
      optionInputList[i].map((v) => !v.optional),
    );
    table.fields.forEach((f) => {
      fieldErrors[f.field_id] = "";
    });
  };

  const removeField = (i: number): void => {
    table.fields.splice(i, 1);
  };

  let removedOGFields = $derived(
    originalTable.fields.filter((f: Field) =>
      table.fields.every((g: Field) => g.field_id !== f.field_id),
    ),
  );

  const restoreField = (i: number): void => {
    table.fields.push($state.snapshot(removedOGFields[i]));
  };

  let optionalCheckboxStates = $state([] as boolean[][]);
  optionalCheckboxStates = optionInputList.map((val) =>
    val.map((v) => !v.optional),
  );
  $inspect(optionalCheckboxStates);

  let fieldErrors = $state([] as string[]);
  table.fields.forEach((f) => {
    fieldErrors[f.field_id] = "";
  });

  let metadataError = $state("");

  let newFields = $derived(
    table.fields.filter((f) =>
      originalTable.fields.every((h) => f.field_id !== h.field_id),
    ),
  );
  let moddedFields = $derived(
    table.fields.filter((f) =>
      originalTable.fields.some(
        (h) => f.field_id === h.field_id && !recursiveCompare(f, h),
      ),
    ),
  );
  const saveFields = () => {
    let promises = [];

    showConfirmScreen = false;

    // modify table name/description
    if (
      table.table.name !== originalTable.table.name ||
      table.table.description !== originalTable.table.description
    ) {
      promises.push(
        putTable(table.table)
          .then((response: Table) => {
            originalTable.table.name = response.name;
            originalTable.table.description = response.description;
            metadataError = "";
            return { ok: true };
          })
          .catch(() => ({ ok: false })),
      );
    }

    // create new fields
    newFields.forEach((field) => {
      promises.push(
        postField(field)
          .then((response: Field) => {
            let newField = response;
            originalTable.fields.push(newField);
            table.fields[
              table.fields.findIndex((f) => f.field_id === field.field_id)
            ].field_id = newField.field_id;
            fieldErrors[field.field_id] = "";
            return { ok: true };
          })
          .catch((e: APIError) => {
            let text = e.body as string;
            fieldErrors[field.field_id] = text;

            return { ok: false };
          }),
      );
    });

    // modify existing fields
    moddedFields.forEach((field) => {
      promises.push(
        putField(field)
          .then((response: Field) => {
            originalTable.fields[
              originalTable.fields.findIndex(
                (f) => f.field_id === field.field_id,
              )
            ] = response;
            fieldErrors[field.field_id] = "";
            return { ok: true };
          })
          .catch((e: APIError) => {
            let text = e.body as string;
            fieldErrors[field.field_id] = text;
            return { ok: false };
          }),
      );
    });

    // delete fields
    for (const field of removedOGFields) {
      promises.push(
        deleteField(field)
          .then(() => {
            originalTable.fields.splice(
              originalTable.fields.findIndex(
                (f) => f.field_id === field.field_id,
              ),
              1,
            );
            return { ok: true };
          })
          .catch(() => {
            fieldErrors[field.field_id] = "Could not delete";
            return { ok: false };
          }),
      );
    }

    // quit or reload
    Promise.allSettled(promises).then((results) => {
      if (results.every((r) => r.status == "fulfilled" && r.value.ok)) {
        on_save();
      }
    });
  };

  const recursiveCompare = (a: any, b: any): boolean => {
    if (typeof a !== typeof b) return false;

    if (a === null) {
      return b === null;
    } else if (Array.isArray(a)) {
      // compare every element
      return a.every((obj, i) => recursiveCompare(obj, b[i]));
    } else if (typeof a === "object") {
      // Check if keys match and if they do, check if objects match
      return (
        recursiveCompare(Object.keys(a).sort(), Object.keys(b).sort()) &&
        Object.keys(a).every((k) => recursiveCompare(a[k], b[k]))
      );
    } else {
      return a === b;
    }
  };

  let showConfirmScreen = $state(false);
  let modalNewFieldLines = $derived(
    newFields.map((f) => `${f.name} (${f.field_kind.type})`),
  );
  let modalModifiedFieldLines = $derived(
    moddedFields.map((f) => {
      let old = originalTable.fields.find(
        (g) => g.field_id === f.field_id,
      ) as Field;
      console.log($state.snapshot(f), $state.snapshot(old));
      return {
        nameAndType:
          f.name !== old.name || f.field_kind.type !== old.field_kind.type
            ? `${old.name} (${old.field_kind.type}) -> ${f.name} (${f.field_kind.type})`
            : "",
        kind: Object.entries(f.field_kind)
          .map((e) => {
            let oldEntry =
              Object.entries(old.field_kind).find((d) => d[0] === e[0]) ?? e;

            console.log(e, Object.entries(old.field_kind), oldEntry);
            if (!recursiveCompare(e[1], oldEntry[1]) && e[0] !== "type") {
              return `${f.name} [${oldEntry[0]}] ${oldEntry[1]} -> ${e[1]}`;
            } else {
              return "";
            }
          })
          .filter((e) => e !== ""),
      };
    }),
  );
  let modalDeletedFieldLines = $derived(
    removedOGFields.map((f) => `${f.name} (${f.field_kind.type})`),
  );
  const openConfirmationModal = () => {
    showConfirmScreen = true;
  };

  let deleteTableConfirmation = $state(false);
</script>

<div class="w-full">
  <!-- Top bar -->
  <label for="name-input">Name: </label>
  <input
    id="name-input"
    bind:value={table.table.name}
    class="text-lg font-bold mb-3"
  />
  <label for="decsription-input">Description: </label>
  <input
    id="description-input"
    bind:value={table.table.description}
    class="text-lg font-bold mb-3"
  />
  {#if metadataError !== ""}
    <p class="text-red-500">{metadataError}</p>
  {/if}
  <button
    class={[
      "rounded-md px-2 py-1 transition",
      !deleteTableConfirmation && "bg-white hover:bg-gray-100",
      deleteTableConfirmation && "bg-red-400 hover:bg-red-500 ",
    ]}
    onclick={deleteTableConfirmation
      ? delete_table
      : () => {
          deleteTableConfirmation = true;
        }}
    onfocusout={() => {
      deleteTableConfirmation = false;
    }}>{deleteTableConfirmation ? "Confirm delete" : "Delete Table"}</button
  >

  <!-- Fields  -->
  <div class="flex items-stretch w-full flex-nowrap overflow-scroll">
    {#if table.fields.length === 0}
      <button
        class="p-12 text-center text-black text-3xl transition-all rounded-lg border-black border-2 border-dashed"
        onclick={() => addField(0)}
        aria-label="add field">+</button
      >
    {:else}
      <button
        class="p-4 hover:p-12 text-center text-transparent hover:text-black text-base hover:text-3xl transition-all"
        onclick={() => addField(0)}
        aria-label="add field">+</button
      >
    {/if}
    {#each table.fields as field, i}
      <div
        class="bg-white border-2 w-80 border-gray-400 p-3 rounded-lg flex flex-col justify-between"
      >
        <input bind:value={table.fields[i].name} />
        {#each optionInputList[i] as optionInput, j}
          <div class="my-2">
            <div class="flex items-center">
              {#if optionInput.optional}
                <input
                  class="mr-2"
                  type="checkbox"
                  bind:checked={() => optionalCheckboxStates[i][j],
                  (val) => {
                    optionalCheckboxStates[i][j] = val;
                    if (!val)
                      delete (table.fields[i].field_kind as any)[
                        optionInput.name
                      ];
                  }}
                />
              {/if}
              <VariableInput
                innerClass={[
                  "w-24",
                  !optionalCheckboxStates && "text-gray-300 border-gray-300",
                ]}
                params={optionInput}
                disabled={!optionalCheckboxStates[i][j]}
                id={optionInput.label + i}
              />
            </div>
          </div>
        {/each}
        <button
          onclick={() => removeField(i)}
          class="rounded-md self-center bg-red-400 hover:bg-red-500 px-2 py-1 transition"
          >Remove</button
        >
        {#if fieldErrors[field.field_id] !== ""}
          <div class="rounded-lg text-red-500">
            {fieldErrors[field.field_id]}
          </div>
        {/if}
      </div>
      <button
        class="p-4 hover:p-12 text-center text-transparent hover:text-black text-base hover:text-3xl transition-all"
        onclick={() => addField(i)}
        aria-label="add field">+</button
      >
    {/each}
    {#each removedOGFields as field, i}
      <div
        class="p-3 border-2 border-gray-400 border-dashed rounded-lg flex flex-col justify-between gap-2"
      >
        <p class="font-bold">{field.name} ({field.field_kind.type})</p>
        <button
          class="py-1 px-2 border-2 border-gray-400 hover:bg-gray-400 border-dashed rounded-lg transition"
          onclick={() => restoreField(i)}>Restore</button
        >
      </div>
      {#if i < removedOGFields.length - 1}
        <button class="p-4 text-center text-transparent text-base" disabled
          >+</button
        >
      {/if}
    {/each}
  </div>

  <!-- Bottom Bar -->
  {#if originalTable !== table}
    <div class="flex items-center justify-center gap-3 mt-4">
      <button
        onclick={openConfirmationModal}
        class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition"
        >Save</button
      >
      <button
        onclick={on_save}
        class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition"
        >Cancel</button
      >
    </div>
  {/if}
</div>

<div
  class={[
    "z-10 size-full fixed top-0 left-0 bg-black/25 flex justify-center items-center",
    !showConfirmScreen && "hidden",
  ]}
>
  <div class="bg-white rounded-lg p-3">
    <h2 class="w-full font-bold text-center">Edit Summary</h2>
    {#if table.table.name !== originalTable.table.name}
      <p>
        <span class="font-bold">Changed Title:</span> "{originalTable.table
          .name}" -&gt "{table.table.name}"
      </p>
    {/if}
    {#if table.table.description !== originalTable.table.description}
      <p>
        <span class="font-bold">Changed Description:</span> "{originalTable
          .table.description}" -&gt "{table.table.description}"
      </p>
    {/if}
    {#each modalNewFieldLines as line}
      <p><span class="font-bold">Added Field:</span> {line}</p>
    {/each}
    {#each modalModifiedFieldLines as moddedField}
      {#if moddedField.nameAndType}
        <p>
          <span class="font-bold">Change Field:</span>
          {moddedField.nameAndType}
        </p>
      {/if}
      {#each moddedField.kind as line}
        <p><span class="font-bold">Change Field Property:</span> {line}</p>
      {/each}
    {/each}
    {#each modalDeletedFieldLines as line}
      <p>
        <span class="font-bold text-red-500">[!]</span>
        <span class="font-bold">Delete Field:</span>
        {line}
      </p>
    {/each}
    <div class="flex justify-center items-center gap-2 mt-2">
      <button
        class="text-center py-1 px-2 rounded bg-gray-100 hover:bg-gray-200 transition"
        onclick={saveFields}>Confirm</button
      >
      <button
        class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition"
        onclick={() => {
          showConfirmScreen = false;
        }}>Cancel</button
      >
    </div>
  </div>
</div>

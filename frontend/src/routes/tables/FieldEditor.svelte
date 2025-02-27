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
    type DecimalOptions,
    type DateTimeOptions,
    type IntervalOptions,
    type WebLinkOptions,
    type EmailOptions,
    type EnumerationOptions,
    type ImageOptions,
    type FileOptions,
    type FieldOptions,
  } from "$lib/types.d.js";

  import { API_URL } from "$lib/api.d.js";

  let { table_prop, on_save } = $props();

  let originalTable: DataTable = $state({
    table: table_prop,
    fields: [],
    entries: [],
  });

  let table = $state($state.snapshot(originalTable));

  const loadFields = () => {
    fetch(`${API_URL}/tables/${table_prop.table_id}/fields`)
      .then(r => r.json())
      .then(j => {
        originalTable.fields = j;
        table = $state.snapshot(originalTable);
        optionalCheckboxStates = optionInputList.map(val => val.map(v => !v.optional));
      })
  }
  loadFields()
  const fieldTypes = Object.values(FieldType);

  type InputType =
    | "button"
    | "color"
    | "date"
    | "datetime-local"
    | "email"
    | "file"
    | "hidden"
    | "image"
    | "month"
    | "number"
    | "password"
    | "radio"
    | "range"
    | "reset"
    | "search"
    | "submit"
    | "tel"
    | "text"
    | "time"
    | "url"
    | "week";

  type OptionInput =
    | {
        name: string;
        label: string;
        type: InputType;
        optional: boolean;
        bindSetter: (val: any) => void;
        bindGetter: () => string | boolean | number;
      }
    | {
        name: string;
        label: string;
        type: "select";
        optional: boolean;
        selectOptions: string[];
        bindSetter: (val: any) => void;
        bindGetter: () => string | boolean | number;
      }
  | {
    name: string;
    label:string;
    type: "checkbox";
    optional: boolean;
    bindSetter: (val: any) => void;
    bindGetter: () => boolean;
  };

  const getTypeOptionInput = (i: number): OptionInput => {
    return {
      name: "type",
      label: "Type",
      type: "select",
      optional:  false,
      selectOptions: fieldTypes,
      bindGetter: () => {
        return table.fields[i].options.type
      },
      bindSetter: (val: FieldType) => {
        // swap out field option if type change
        if(val != table.fields[i].options.type) {
          switch(val){
            case FieldType.Text:
              table.fields[i].options = {
                type: val,
                is_required: true
              };
              break;
            case FieldType.Integer:
              table.fields[i].options = {
                type: val,
                is_required: true
              };
              break;
            case FieldType.Decimal:
              table.fields[i].options = {
                type: val,
                is_required: true,

                scientific_notation: true
              };
              break;
            case FieldType.Money:
              table.fields[i].options = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Progress:
              table.fields[i].options = {
                type: val,
                total_steps: 100
              };
              break;
            case FieldType.DateTime:
              table.fields[i].options = {
                type: val,
                is_required: true,

                date_time_format: "YYYY-MM-DD"
              };
              break;
            case FieldType.Interval:
              table.fields[i].options = {
                type: val,
                is_required: true
              };
              break;
            case FieldType.WebLink:
              table.fields[i].options = {
                type: val,
                is_required: true
              };
              break;
            case FieldType.Email:
              table.fields[i].options = {
                type: val,
                is_required: true
              };
              break;
            case FieldType.Checkbox:
              table.fields[i].options = {
                type: val
              };
              break;
            case FieldType.Enumeration:
              table.fields[i].options = {
                type: val,
                is_required: true,
                values: new Map([["Item", 0]]),
                default: 0
              };
              break;
            case FieldType.Image:
              table.fields[i].options = {
                type: val,
                is_required: true
              };
              break;
            case FieldType.File:
              table.fields[i].options = {
                type: val,
                is_required: true
              };
              break;
          }
        }
      }
    }
  };

  type RequirableOptions =TextOptions
  | IntegerOptions
  | DecimalOptions
  | MoneyOptions
  | DateTimeOptions
  | IntervalOptions
  | WebLinkOptions
  | EmailOptions
  | EnumerationOptions
  | ImageOptions
  | FileOptions; 

  const getRequiredOptionInput = (i: number): OptionInput => {
    return {
      name: "is_required",
      label: "Is Required",
      type: "checkbox",
      optional: false,
      bindGetter: () => {
        return (table.fields[i].options as RequirableOptions).is_required;
      },
      bindSetter: (val: boolean) => {
        (table.fields[i].options as RequirableOptions).is_required = val;
      }
    }
  };

  const optionInputList = $derived(table.fields.map((f: Field, i: number): OptionInput[] => {
    switch(f.options.type) {
      case FieldType.Text:
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
        ];
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
              return (table.fields[i].options as IntegerOptions).range_start ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as IntegerOptions).range_start = val;
            }
          },{
            name: "range_end",
            label: "Range end",
            type: "number",
            optional: true,
            bindGetter: () => {
              return (table.fields[i].options as IntegerOptions).range_end ?? 100;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as IntegerOptions).range_end = val;
            }
          }
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
              return (table.fields[i].options as DecimalOptions).range_start ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as DecimalOptions).range_start = val;
            }
          },
          {
            name: "range_end",
            label: "Range end",
            type: "number",
            optional: true,
            bindGetter: () => {
              return (table.fields[i].options as DecimalOptions).range_end ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as DecimalOptions).range_end = val;
            }
          },
          {
            name: "scientific_notation",
            label: "Scientific notation",
            type: "checkbox",
            optional: false,
            bindGetter: () => {
              return (table.fields[i].options as DecimalOptions).scientific_notation;
            },
            bindSetter: (val: boolean) => {
              (table.fields[i].options as DecimalOptions).scientific_notation = val;
            }
          },
          {
            name: "number_precision",
            label: "Number Precision",
            type: "number",
            optional: true,
            bindGetter: () => {
              return (table.fields[i].options as DecimalOptions).number_precision ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as DecimalOptions).number_precision = val;
            }
          },{
            name: "number_scale",
            label: "Number Scale",
            type: "number",
            optional: true,
            bindGetter: () => {
              return (table.fields[i].options as DecimalOptions).number_scale ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as DecimalOptions).number_scale = val;
            }
          }
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
              return (table.fields[i].options as MoneyOptions).range_start ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as MoneyOptions).range_start = val;
            }
          },
          {
            name: "range_end",
            label: "Range end",
            type: "number",
            optional: true,
            bindGetter: () => {
              return (table.fields[i].options as MoneyOptions).range_end ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as MoneyOptions).range_end = val;
            }
          },
        ];
      case FieldType.Progress:
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
          {
            name: "total_steps",
            label: "Total steps",
            type: "number",
            optional: false,
            bindGetter: () => {
              return (table.fields[i].options as ProgressOptions).total_steps ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as ProgressOptions).total_steps = val;
            }
          }
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
              return (table.fields[i].options as DateTimeOptions).range_start?.toISOString() ?? (new Date()).toString();
            },
            bindSetter: (val: string) => {
              (table.fields[i].options as DateTimeOptions).range_start = new Date(val);
            }
          },
          {
            name: "range_end",
            label: "Range end",
            type: "datetime-local",
            optional: true,
            bindGetter: () => {
              return (table.fields[i].options as DateTimeOptions).range_end?.toISOString() ?? (new Date()).toString();
            },
            bindSetter: (val: string) => {

              (table.fields[i].options as DateTimeOptions).range_end = new Date(val);
            }
          },
          {
            name: "date_time_format",
            label: "DateTime format",
            type: "text",
            optional: false,
            bindGetter: () => {
              return (table.fields[i].options as DateTimeOptions).date_time_format;
            },
            bindSetter: (val: string) => {

              (table.fields[i].options as DateTimeOptions).date_time_format = val;
            }
          },
        ];
      case FieldType.Interval:
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
        ];
      case FieldType.WebLink:
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
        ];
      case FieldType.Email:
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
        ];
      case FieldType.Checkbox: 
        return [
          getTypeOptionInput(i),
        ];
      case FieldType.Enumeration:
        // TODO: Add map input somehow
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
        ];
      case FieldType.Image:
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
        ];
      case FieldType.File:
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
        ];
      default:
        return [];
    }
  }));

  const addField = (i: number): void => {
    let j = 1;
    let newFieldName = "New Field " + j;
    while(table.fields.some((f: Field) => f.name === newFieldName)) {
      newFieldName = "New Field " + ++j;
    }

    let newField: Field = {
      // These first three fields should be set upon creation by the backend and are merely defined here to satisfy Typescript
      table_id: -1,
      user_id: -1, 
      field_id: -1,

      name: newFieldName,
      options: {
        type: FieldType.Text,
        is_required: true
      }
    };

    table.fields.splice(i+1,0,newField);
    optionalCheckboxStates = optionInputList.map(val => val.map(v => !v.optional));
  }
  
  const removeField = (i: number): void => {
    table.fields.splice(i, 1);
  }

  let removedOGFields = $derived(originalTable.fields.filter((f: Field) => table.fields.every((g: Field) => g.field_id !== f.field_id)));

  const restoreField = (i: number): void => {
    table.fields.push($state.snapshot(removedOGFields[i]));
  }
  
  $inspect(table, originalTable, removedOGFields)

  let optionalCheckboxStates = $state([] as boolean[][]);
  optionalCheckboxStates = optionInputList.map(val => val.map(v => !v.optional));



  const saveFields = () => {
    let promises = []


    // TODO: reduce field objects to minimum required request bodies AND/OR refactor fetches into their own functions

    // create new fields
    let newFields = table.fields.filter(f => originalTable.fields.every(h => f.field_id !== h.field_id))
    for(const field of newFields) {
      promises.push(fetch(`${API_URL}/tables/${table_prop.table_id}/fields`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          name: field.name,
          options: field.options})
      }));
    }

    // modify existing fields
    let moddedFields = table.fields.filter(f => originalTable.fields.some(h => f.field_id === h.field_id && !recursiveCompare(f, h)));
    for(const field of moddedFields){
      promises.push(fetch(`${API_URL}/tables/${table_prop.table_id}/fields/${field.field_id}`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          name: field.name,
          options: field.options
        })
      }));
    }

    // delete fields
    for(const field of removedOGFields){
      promises.push(fetch(`${API_URL}/tables/${table_prop.table_id}/fields/${field.field_id}`, {
          method: "DELETE"
      }));
    }


    // quit or reload
    Promise.allSettled(promises).then((results) => {
      if(results.every(r => r.status == "fulfilled" && r.value.ok)){
        on_save();
      } else {
        loadFields();
      }
    })
  }

  const recursiveCompare= (a: any, b: any):boolean => {
    console.log(a, b);
    if (typeof a !== typeof b) return false;

    if(a === null){
      return b === null;
    } else if(Array.isArray(a)){
      // compare every element
      return a.every((obj, i) => recursiveCompare(obj, b[i]));
    } else if(typeof a === "object") {
      // Check if keys match...                                                 ...and if they do, check if objects match
      return recursiveCompare(Object.keys(a).sort(), Object.keys(b).sort()) && Object.keys(a).every(k => recursiveCompare(a[k], b[k]));
    } else {
      return a === b;
    }
  }
</script>

<div class="w-full">
  <!-- Top bar -->
  <input bind:value={table.table.name} class="text-lg font-bold mb-3" />
  
  <!-- Fields  -->
  <div class="flex items-stretch w-full flex-nowrap overflow-scroll">
    {#if table.fields.length === 0}
      <button class="p-12 text-center text-black text-3xl transition-all rounded-lg border-black border-2 border-dashed" onclick={() => addField(0)} aria-label="add field">+</button>
    {:else}
      <button class="p-4 hover:p-12 text-center text-transparent hover:text-black text-base hover:text-3xl transition-all" onclick={() => addField(0)} aria-label="add field">+</button>
    {/if}
    {#each table.fields as field, i}
      <div class="bg-white border-2 border-gray-400 p-3 rounded-lg flex flex-col justify-between">
        <input bind:value={table.fields[i].name} />
          {#each optionInputList[i] as optionInput, j}
            <div class="flex items-center my-2">
              {#if optionInput.optional}
                <input class="mr-2" type="checkbox" bind:checked={() => optionalCheckboxStates[i][j], (val) => {optionalCheckboxStates[i][j] = val; if(!val) delete (table.fields[i].options as any)[optionInput.name]}}/>
              {/if}
              <label class={["mr-2 min-w-28", !optionalCheckboxStates[i][j] && "text-gray-300"]} for={optionInput.label + i}>{optionInput.label}:</label>
              {#if optionInput.type === "select"}
                <select disabled={!optionalCheckboxStates[i][j]} id={optionInput.label + i} bind:value={optionInput.bindGetter, optionInput.bindSetter}>
                  {#each optionInput.selectOptions as opt}
                    <option>{opt}</option>
                  {/each}
                </select>
              {:else if optionInput.type === "checkbox"}
                <input class={[!optionalCheckboxStates[i][j] && "text-gray-300 border-gray-300"]} disabled={!optionalCheckboxStates[i][j]} id={optionInput.label + i} type="checkbox" bind:checked={optionInput.bindGetter, optionInput.bindSetter} />
              {:else}
                <input class={["w-24", !optionalCheckboxStates[i][j] && "text-gray-300 border-gray-300"]} disabled={!optionalCheckboxStates[i][j]} id={optionInput.label + i} type={optionInput.type} bind:value={optionInput.bindGetter, optionInput.bindSetter} />
              {/if}
            </div>
          {/each}
        <button onclick={() => removeField(i)} class="rounded-md self-center bg-red-400 hover:bg-red-500 px-2 py-1 transition">Remove</button>
      </div>
        <button class="p-4 hover:p-12 text-center text-transparent hover:text-black text-base hover:text-3xl transition-all" onclick={() => addField(i)} aria-label="add field">+</button>
    {/each}
    {#each removedOGFields as field, i}
      <div class="p-3 border-2 border-gray-400 border-dashed rounded-lg flex flex-col justify-between gap-2 ">
        <p class="font-bold">{field.name} ({field.options.type})</p>
        <button class="py-1 px-2 border-2 border-gray-400 hover:bg-gray-400 border-dashed rounded-lg transition" onclick={() => restoreField(i)}>Restore</button>
      </div>
      {#if i < removedOGFields.length-1}
        <button class="p-4 text-center text-transparent text-base" disabled>+</button>
      {/if}
    {/each}
  </div>
  
  <!-- Bottom Bar -->
  {#if originalTable !== table}
    <div class="flex items-center justify-center gap-3">
      <button onclick={saveFields} class="text-center py-1 px-2 rounded bg-white hover:bg-gray-100 transition">Save</button>
      <button class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition">Cancel</button>
    </div>
  {/if}
</div>

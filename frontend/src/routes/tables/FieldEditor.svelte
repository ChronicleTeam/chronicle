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
    type InputParameters,
    parseJSONTable
  } from "$lib/types.d.js";
  import VariableInput from "$lib/components/VariableInput.svelte";
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
        originalTable.fields = parseJSONTable({table: {}, fields: j, entries: []}).fields;
        table = $state.snapshot(originalTable);
        optionalCheckboxStates = optionInputList.map(val => val.map(v => !v.optional));
        table.fields.forEach(f => {fieldErrors[f.field_id] = ""});
      })
  }
  loadFields()
  const fieldTypes = Object.values(FieldType);


  type OptionInputParameters = InputParameters & {
    optional: boolean,
    name: string,
  }

  const getTypeOptionInput = (i: number): OptionInputParameters => {
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
                values: {} as {[key:number]:string},
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
          optionalCheckboxStates[i] = optionInputList[i].map(v => !v.optional);
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

  const getRequiredOptionInput = (i: number): OptionInputParameters => {
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

  const optionInputList = $derived(table.fields.map((f: Field, i: number): OptionInputParameters[] => {
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
              return (table.fields[i].options as DateTimeOptions).range_start?.toISOString().substring(0,19) ?? (new Date()).toISOString().substring(0,19);
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
              return (table.fields[i].options as DateTimeOptions).range_end?.toISOString().substring(0,19) ?? (new Date()).toISOString().substring(0,19);
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
          {
            name: "values",
            label: "values",
            type: "textarea",
            optional: false,
            bindGetter:() => {
              return Object.entries((table.fields[i].options as EnumerationOptions).values)
                  .map((entry)=> entry[0].toString() +":"+entry[1])
                  .join("\n")
            },
            bindSetter:(val: string) => {
              (table.fields[i].options as EnumerationOptions).values = Object.fromEntries(
                  val.split("\n")
                  .map(line => {let entry = line.split(":", 2); return entry.length < 2 ? [entry[0], ""] : entry}) // split
                  .map((entry: string[]) => [parseInt(entry[0]), entry[1]]) // parse
                  .map((entry, i, arr) => {
                    if(isNaN(entry[0] as number)){
                      let i = 0;
                      while(arr.some(e => e[0] === i)){
                        i++
                      }
                      return [i, entry[1]]
                    } else {
                      return entry
                    }
                  } // catch NaN
                  ) as [number, string][]);
            }
          }
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

    let id = -1;
    while(table.fields.some(f => f.field_id === id)){
      id--;
    }
    let newField: Field = {
      // These first three fields should be set upon creation by the backend and are merely defined here to satisfy Typescript
      table_id: -1,
      user_id: -1, 
      field_id: id,

      name: newFieldName,
      options: {
        type: FieldType.Text,
        is_required: true
      }
    };

    table.fields.splice(i+1,0,newField);
    optionalCheckboxStates.splice(i+1, 0, optionInputList[i].map(v => !v.optional));
    table.fields.forEach(f => {fieldErrors[f.field_id] = ""});
  }
  
  const removeField = (i: number): void => {
    table.fields.splice(i, 1);
  }

  let removedOGFields = $derived(originalTable.fields.filter((f: Field) => table.fields.every((g: Field) => g.field_id !== f.field_id)));

  const restoreField = (i: number): void => {
    table.fields.push($state.snapshot(removedOGFields[i]));
  }
  

  let optionalCheckboxStates = $state([] as boolean[][]);
  optionalCheckboxStates = optionInputList.map(val => val.map(v => !v.optional));
  $inspect(optionalCheckboxStates);

  let fieldErrors = $state([] as string[]);
  table.fields.forEach(f => {fieldErrors[f.field_id] = ""});
  
  let metadataError = $state("");

  const saveFields = () => {
    let promises = []


    // TODO: reduce field objects to minimum required request bodies AND/OR refactor fetches into their own functions

    // modify table name/description
    if(table.table.name !== originalTable.table.name || table.table.description !== originalTable.table.description){
      promises.push(fetch(`${API_URL}/tables/${table_prop.table_id}`,{
        method: "PUT",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          name: table.table.name,
          description: table.table.description
        })
      }).then(async response => {
          if(response.status === 200){
            let metadata = await response.json()
            originalTable.table.name = metadata.name;
            originalTable.table.description = metadata.description;
            metadataError = "";
            return {ok: true}
          } else if(response.status === 422) {
            metadataError = await response.text();
          }
            return {ok: false}
        }));
    }

    // create new fields
    let newFields = table.fields.filter(f => originalTable.fields.every(h => f.field_id !== h.field_id))
    newFields.forEach((field, i) => {
      promises.push(fetch(`${API_URL}/tables/${table_prop.table_id}/fields`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          name: field.name,
          options: field.options})
      }).then(async response => {
          if(response.status === 200){
            let newField = await response.json()
            originalTable.fields.push(newField)
            table.fields[table.fields.findIndex(f => f.field_id ===field.field_id)].field_id = newField.field_id;
            fieldErrors[field.field_id] = "";
            return {ok: true}
          }else if(response.status === 422){
            let text = await response.text() 
            fieldErrors[field.field_id] = text;
          }
          return {ok: false}
        }));
    })

    // modify existing fields
    let moddedFields = table.fields.filter(f => originalTable.fields.some(h => f.field_id === h.field_id && !recursiveCompare(f, h)));
    moddedFields.forEach((field, i) => {
      promises.push(fetch(`${API_URL}/tables/${table_prop.table_id}/fields/${field.field_id}`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          name: field.name,
          options: field.options
        })
      }).then(async response => {
          if(response.status === 200){
            originalTable.fields[originalTable.fields.findIndex(f => f.field_id === field.field_id)] = await response.json()
            fieldErrors[field.field_id] = "";
            return {ok: true};
          }else if(response.status === 422){
            let text = await response.text() 
            fieldErrors[field.field_id] = text;
          }
          return {ok: false};
        }))
    })

    // delete fields
    for(const field of removedOGFields){
      promises.push(fetch(`${API_URL}/tables/${table_prop.table_id}/fields/${field.field_id}`, {
          method: "DELETE"
      }).then(async response => {
          if(response.status === 200){
            originalTable.fields.splice(originalTable.fields.findIndex(f => f.field_id === field.field_id), 1);
            return {ok: true};
          }
          fieldErrors[field.field_id] = "Could not delete";
          return {ok: false};
        }));
    }


    // quit or reload
    Promise.allSettled(promises).then((results) => {
      if(results.every(r => r.status == "fulfilled" && r.value.ok)){
        on_save();
      } else {
        originalTable.fields = parseJSONTable({table: {}, fields: $state.snapshot(originalTable).fields, entries: []}).fields;
      }
    })
  }

  const recursiveCompare= (a: any, b: any):boolean => {
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
  <label for="name-input">Name: </label>
  <input id="name-input" bind:value={table.table.name} class="text-lg font-bold mb-3" />
  <label for="decsription-input">Description: </label>
  <input id="description-input" bind:value={table.table.description} class="text-lg font-bold mb-3" />
  {#if metadataError !== ""} <p class="text-red-500">{metadataError}</p>{/if}

  <!-- Fields  -->
  <div class="flex items-stretch w-full flex-nowrap overflow-scroll">
    {#if table.fields.length === 0}
      <button class="p-12 text-center text-black text-3xl transition-all rounded-lg border-black border-2 border-dashed" onclick={() => addField(0)} aria-label="add field">+</button>
    {:else}
      <button class="p-4 hover:p-12 text-center text-transparent hover:text-black text-base hover:text-3xl transition-all" onclick={() => addField(0)} aria-label="add field">+</button>
    {/if}
    {#each table.fields as field, i}
      <div class="bg-white border-2 w-80 border-gray-400 p-3 rounded-lg flex flex-col justify-between ">
        <input bind:value={table.fields[i].name} />
          {#each optionInputList[i] as optionInput, j}
            <div class="my-2">
              <div class="flex items-center">
                {#if optionInput.optional}
                  <input class="mr-2" type="checkbox" bind:checked={() => optionalCheckboxStates[i][j], (val) => {optionalCheckboxStates[i][j] = val; if(!val) delete (table.fields[i].options as any)[optionInput.name]}}/>
                {/if}
                <VariableInput innerClass={["w-24", !optionalCheckboxStates && "text-gray-300 border-gray-300"]} params={optionInput} disabled={!optionalCheckboxStates[i][j]} id={optionInput.label+i} />
              </div>
            </div>
          {/each}
        <button onclick={() => removeField(i)} class="rounded-md self-center bg-red-400 hover:bg-red-500 px-2 py-1 transition">Remove</button>
        {#if fieldErrors[field.field_id] !== ""}
        <div class="rounded-lg text-red-500">
          {fieldErrors[field.field_id]}
        </div>
        {/if}
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
      <button onclick={on_save} class="text-center py-1 px-2 rounded bg-red-400 hover:bg-red-500 transition">Cancel</button>
    </div>
  {/if}
</div>

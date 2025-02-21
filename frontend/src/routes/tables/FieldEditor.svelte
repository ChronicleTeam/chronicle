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
  } from "$lib/types.d.js";
    import { optimizeDeps } from "vite";

  let { table_prop } = $props();

  const fieldTypes = Object.values(FieldType);
  let table: DataTable = {
    table: table_prop,
    fields: [
      {
        name: "Project Name",
        options: {
          type: FieldType.Text,
          is_required: true,
        },
      },
      {
        name: "Funding",
        options: {
          type: FieldType.Money,
          is_required: false,
        },
      },
      {
        name: "Members",
        options: {
          type: FieldType.Integer,
          is_required: true,
        },
      },
      {
        name: "Progress",
        options: {
          type: FieldType.Progress,
          total_steps: 100,
        },
      },
    ],
    entries: [],
  };

  // TODO: add setter functions for bind() call such that the field options change type when the field type is changed.
  const fieldTypeSetters = $derived(
    table.fields.map((f: Field, i: number) => {
      return (val: FieldType) => {
        if (table.fields[i].options.type != val) {
          table.fields[i].options.type = val;
          switch (val) {
            case FieldType.Text:
              table.fields[i].options = {
                type: val,
                is_required: true,
              } as TextOptions;
              break;
            case FieldType.Money:
              table.fields[i].options = {
                type: val,
                is_required: true,
              } as MoneyOptions;
              break;
            case FieldType.Integer:
              table.fields[i].options = {
                type: val,
                is_required: true,
              } as IntegerOptions;
              break;
            case FieldType.Progress:
              table.fields[i].options = {
                type: val,
                total_steps: 100,
              } as ProgressOptions;
              break;
          }
        }
      };
    }),
  );

  type InputType =
    | "button"
    | "checkbox"
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
        label: string;
        type: InputType;
        optional: boolean;
        enabled: boolean;
        bindSetter: (val: any) => void;
        bindGetter: () => string | boolean | number;
      }
    | {
        label: string;
        type: "select";
        optional: boolean;
        enabled: boolean;
        selectOptions: string[];
        bindSetter: (val: any) => void;
        bindGetter: () => string | boolean | number;
      };

  const GetOptionInputList = (f: Field): OptionInput[] => {
    return [];
  };

  const getTypeOptionInput = (i: number): OptionInput => {
    return {
      label: "Type",
      type: "select",
      optional:  false,
      enabled: true,
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
      label: "Is Required",
      type: "checkbox",
      optional: false,
      enabled: true,
      bindGetter: () => {
        return (table.fields[i].options as RequirableOptions).is_required;
      },
      bindSetter: (val: boolean) => {
        (table.fields[i].options as RequirableOptions).is_required = val;
      }
    }
  };

  const OptionInputList = $derived(table.fields.map((f: Field, i: number): OptionInput[] => {
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
            label: "Range start",
            type: "number",
            optional: true,
            enabled: false,
            bindGetter: () => {
              return (table.fields[i].options as IntegerOptions).range_start ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as IntegerOptions).range_start = val;
            }
          },{
            label: "Range end",
            type: "number",
            optional: true,
            enabled: false,
            bindGetter: () => {
              return (table.fields[i].options as IntegerOptions).range_start ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as IntegerOptions).range_start = val;
            }
          }
        ];
      case FieldType.Decimal:
        return [
          getTypeOptionInput(i),
          getRequiredOptionInput(i),
          {
            label: "Scientific notation",
            type: "checkbox",
            optional: false,
            enabled: true,
            bindGetter: () => {
              return (table.fields[i].options as DecimalOptions).scientific_notation;
            },
            bindSetter: (val: boolean) => {
              (table.fields[i].options as DecimalOptions).scientific_notation = val;
            }
          },
          {
            label: "Number Precision",
            type: "number",
            optional: true,
            enabled: false,
            bindGetter: () => {
              return (table.fields[i].options as DecimalOptions).number_precision ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as DecimalOptions).number_precision = val;
            }
          },{
            label: "Number Scale",
            type: "number",
            optional: true,
            enabled: false,
            bindGetter: () => {
              return (table.fields[i].options as DecimalOptions).number_scale ?? 0;
            },
            bindSetter: (val: number) => {
              (table.fields[i].options as DecimalOptions).number_scale = val;
            }
          }
        ];
      case FieldType.Money: // TODO
      case FieldType.Progress: // TODO
      case FieldType.DateTime: // TODO
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
      case FieldType.Checkbox: // TODO
      case FieldType.Enumeration: // TODO
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


</script>

<div class="w-full">
  <!-- Top bar -->
  <input bind:value={table.table.name} class="text-lg font-bold mb-3" />
  <!-- Fields  -->
  <div class="flex items-stretch gap-5 w-full flex-nowrap overflow-scroll">
    {#each table.fields as field, i}
      <div class="bg-white p-3 rounded-lg">
        <input bind:value={table.fields[i].name} />
        <div class="flex items-center">
          <label for="typeSelect" class="mr-2">Type:</label>
          <select
            id="typeSelect"
            class="my-2"
            bind:value={table.fields[i].options.type}
          >
            {#each fieldTypes as fieldType}
              <option value={fieldType}>{fieldType}:</option>
            {/each}
          </select>
          {#each GetOptionInputList(field) as optionInput}
            <div class="flex items-center">
              {#if optionInput.optional}
                <input type="checkbox" bind:value={optionInput.enabled}/>
              {/if}
              <label for={optionInput.label}>{optionInput.label}</label>
              {#if optionInput.type === "select"}
                <select disabled={optionInput.enabled} id={optionInput.label}>
                  {#each optionInput.selectOptions as opt}
                    <option>{opt}</option>
                  {/each}
                </select>
              {:else}
                <input id={optionInput.label} type={optionInput.type} />
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</div>

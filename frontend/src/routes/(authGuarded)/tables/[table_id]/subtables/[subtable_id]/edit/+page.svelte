<script lang="ts">
  import {
    type TableData,
    type Field,
    type Cell,
    type TextKind,
    type MoneyKind,
    type IntegerKind,
    type ProgressKind,
    FieldType,
    type DecimalKind,
    type DateTimeKind,
    type WebLinkKind,
    type EnumerationKind,
    type InputParameters,
    type Table,
    typeToStr,
  } from "$lib/types";
  import VariableInput from "$lib/components/VariableInput.svelte";
  import ConfirmButton from "$lib/components/ConfirmButton.svelte";
  import {
    patchTable,
    getFields,
    postField,
    patchField,
    deleteField,
    type APIError,
    postCreateTable,
    deleteTable,
    getTableData,
  } from "$lib/api";
  import { onMount } from "svelte";
  import { goto, invalidate, invalidateAll } from "$app/navigation";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();

  //
  // Constants and types
  //

  // field type values
  const fieldTypes = Object.values(FieldType);

  // extends the InputParameters type
  type FieldKindInputParameters =
    | (InputParameters & {
        optional: false;
        name: string;
      })
    | (InputParameters & {
        optional: true;
        name: string;
        default: Cell;
      });

  // field_kinds that have an is_required entry
  type RequirableKind =
    | TextKind
    | IntegerKind
    | DecimalKind
    | MoneyKind
    | DateTimeKind
    | WebLinkKind
    | EnumerationKind;

  //
  // State variables
  //

  // if the current table has a parent
  let isSubtable = $state(data.table_prop.parent_id != null);

  // old table state (as it is in the backend) and new table state (as the user is changing it to be)
  let table: {
    old: TableData; // original table
    new: TableData; // table undergoing modifications
  } = $state({
    old: {
      table: data.table_prop,
      fields: [],
      entries: [],
      children: [],
    },
    new: {
      table: $state.snapshot(data.table_prop),
      fields: [],
      entries: [],
      children: [],
    },
  });

  // keeps track of changes between table.old and table.new
  let changes: {
    fields: {
      removed: Field[];
      modified: Field[];
      added: Field[];
    };
    subtables: {
      removed: TableData[];
      modified: TableData[];
      added: TableData[];
    };
  } = $derived({
    fields: {
      removed: table.old.fields.filter((f: Field) =>
        table.new.fields.every((g: Field) => g.field_id !== f.field_id),
      ),
      modified: table.new.fields.filter((f) =>
        table.old.fields.some(
          (h) => f.field_id === h.field_id && !recursiveCompare(f, h),
        ),
      ),
      added: table.new.fields.filter((f) =>
        table.old.fields.every((h) => f.field_id !== h.field_id),
      ),
    },

    subtables: {
      removed: table.old.children.filter((t) =>
        table.new.children.every((u) => t.table.table_id !== u.table.table_id),
      ),
      modified: table.new.children.filter((t) =>
        table.old.children.some(
          (u) =>
            t.table.table_id === u.table.table_id && !recursiveCompare(t, u),
        ),
      ),

      added: table.new.children.filter((t) =>
        table.old.children.every((u) => t.table.table_id !== u.table.table_id),
      ),
    },
  });

  // error fields
  let errors: {
    fields: string[];
    subtables: string[];
    metadata: string;
  } = $state({
    fields: [],
    subtables: [],
    metadata: "",
  });

  // the central table which represents the inputs for the editable field_kind parameters
  const fieldKindInputList = $derived(
    table.new.fields.map((f: Field, i: number): FieldKindInputParameters[] => {
      switch (f.field_kind.type) {
        case FieldType.Text:
          return [getTypeFieldKindInput(i), getRequiredFieldKindInput(i)];
        case FieldType.Integer:
          return [
            getTypeFieldKindInput(i),
            getRequiredFieldKindInput(i),
            {
              name: "range_start",
              label: "Range start",
              type: "number",
              optional: true,
              default: 0,
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as IntegerKind).range_start ??
                  0
                );
              },
              bindSetter: (val: number) => {
                (table.new.fields[i].field_kind as IntegerKind).range_start =
                  val;
              },
            },
            {
              name: "range_end",
              label: "Range end",
              type: "number",
              optional: true,
              default: 100,
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as IntegerKind).range_end ??
                  100
                );
              },
              bindSetter: (val: number) => {
                (table.new.fields[i].field_kind as IntegerKind).range_end = val;
              },
            },
          ];
        case FieldType.Decimal:
          return [
            getTypeFieldKindInput(i),
            getRequiredFieldKindInput(i),
            {
              name: "range_start",
              label: "Range start",
              type: "number",
              optional: true,
              default: 0,
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as DecimalKind).range_start ??
                  0
                );
              },
              bindSetter: (val: number) => {
                (table.new.fields[i].field_kind as DecimalKind).range_start =
                  val;
              },
              step: Math.pow(10, -(f.field_kind.number_scale ?? 10)),
            },
            {
              name: "range_end",
              label: "Range end",
              type: "number",
              optional: true,
              default: 100,
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as DecimalKind).range_end ??
                  100
                );
              },
              bindSetter: (val: number) => {
                (table.new.fields[i].field_kind as DecimalKind).range_end = val;
              },
              step: Math.pow(10, -(f.field_kind.number_scale ?? 10)),
            },
            {
              name: "scientific_notation",
              label: "Scientific notation",
              type: "checkbox",
              optional: false,
              bindGetter: () => {
                return (table.new.fields[i].field_kind as DecimalKind)
                  .scientific_notation;
              },
              bindSetter: (val: boolean) => {
                (
                  table.new.fields[i].field_kind as DecimalKind
                ).scientific_notation = val;
              },
            },
            {
              name: "number_precision",
              label: "Number Precision",
              type: "number",
              optional: true,
              default: 20,
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as DecimalKind)
                    .number_precision ?? 20
                );
              },
              bindSetter: (val: number) => {
                (
                  table.new.fields[i].field_kind as DecimalKind
                ).number_precision = val;
              },
            },
            {
              name: "number_scale",
              label: "Number Scale",
              type: "number",
              optional: true,
              default: 10,
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as DecimalKind)
                    .number_scale ?? 10
                );
              },
              bindSetter: (val: number) => {
                (table.new.fields[i].field_kind as DecimalKind).number_scale =
                  val;
              },
            },
          ];
        case FieldType.Money:
          return [
            getTypeFieldKindInput(i),
            getRequiredFieldKindInput(i),
            {
              name: "range_start",
              label: "Range start",
              type: "number",
              optional: true,
              default: "0.00",
              bindGetter: () => {
                return parseFloat(
                  (table.new.fields[i].field_kind as MoneyKind).range_start ??
                    "0.00",
                );
              },
              bindSetter: (val: number) => {
                (table.new.fields[i].field_kind as MoneyKind).range_start =
                  val.toFixed(2);
              },
              step: 0.01,
            },
            {
              name: "range_end",
              label: "Range end",
              type: "number",
              optional: true,
              default: "100.00",
              bindGetter: () => {
                return parseFloat(
                  (table.new.fields[i].field_kind as MoneyKind).range_end ??
                    "100.00",
                );
              },
              bindSetter: (val: number) => {
                (table.new.fields[i].field_kind as MoneyKind).range_end =
                  val.toFixed(2);
              },
              step: 0.01,
            },
          ];
        case FieldType.Progress:
          return [
            getTypeFieldKindInput(i),
            {
              name: "total_steps",
              label: "Total steps",
              type: "number",
              optional: false,
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as ProgressKind)
                    .total_steps ?? 0
                );
              },
              bindSetter: (val: number) => {
                (table.new.fields[i].field_kind as ProgressKind).total_steps =
                  val;
              },
            },
          ];
        case FieldType.DateTime:
          return [
            getTypeFieldKindInput(i),
            getRequiredFieldKindInput(i),
            {
              name: "range_start",
              label: "Range start",
              type: "datetime-local",
              optional: true,
              default: new Date(),
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as DateTimeKind).range_start
                    ?.toISOString()
                    .substring(0, 19) ??
                  new Date().toISOString().substring(0, 19)
                );
              },
              bindSetter: (val: string) => {
                (table.new.fields[i].field_kind as DateTimeKind).range_start =
                  new Date(val);
              },
            },
            {
              name: "range_end",
              label: "Range end",
              type: "datetime-local",
              optional: true,
              default: new Date(),
              bindGetter: () => {
                return (
                  (table.new.fields[i].field_kind as DateTimeKind).range_end
                    ?.toISOString()
                    .substring(0, 19) ??
                  new Date().toISOString().substring(0, 19)
                );
              },
              bindSetter: (val: string) => {
                (table.new.fields[i].field_kind as DateTimeKind).range_end =
                  new Date(val);
              },
            },
            {
              name: "date_time_format",
              label: "DateTime format",
              type: "text",
              optional: false,
              bindGetter: () => {
                return (table.new.fields[i].field_kind as DateTimeKind)
                  .date_time_format;
              },
              bindSetter: (val: string) => {
                (
                  table.new.fields[i].field_kind as DateTimeKind
                ).date_time_format = val;
              },
            },
          ];
        case FieldType.WebLink:
          return [getTypeFieldKindInput(i), getRequiredFieldKindInput(i)];
        case FieldType.Checkbox:
          return [getTypeFieldKindInput(i)];
        case FieldType.Enumeration:
          return [
            getTypeFieldKindInput(i),
            getRequiredFieldKindInput(i),
            {
              name: "values",
              label: "Values",
              type: "textarea",
              optional: false,
              bindGetter: () => {
                return Object.entries(
                  (table.new.fields[i].field_kind as EnumerationKind).values,
                )
                  .map((entry) => entry[0].toString() + ":" + entry[1])
                  .join("\n");
              },
              bindSetter: (val: string) => {
                (table.new.fields[i].field_kind as EnumerationKind).values =
                  Object.fromEntries(
                    val
                      .split("\n")
                      .map((line) => {
                        let entry = line.split(":", 2);
                        return entry.length < 2 ? [entry[0], ""] : entry;
                      }) // split
                      .map((entry: string[]) => [parseInt(entry[0]), entry[1]]) // parse
                      .map((entry, i, arr) => {
                        if (isNaN(entry[0] as number)) {
                          // catch NaN
                          let i = 0;
                          while (arr.some((e) => e[0] === i)) {
                            i++;
                          }
                          return [i, entry[1]];
                        } else {
                          return entry;
                        }
                      }) as [number, string][],
                  );
              },
            },
            {
              name: "default_value",
              label: "Default value",
              type: "number",
              optional: false,
              bindGetter: () =>
                (table.new.fields[i].field_kind as EnumerationKind)
                  .default_value,
              bindSetter: (val: number) => {
                (
                  table.new.fields[i].field_kind as EnumerationKind
                ).default_value = val;
              },
            },
          ];
        default:
          return [];
      }
    }),
  );

  // this controls which inputs are disabled in the Field editor, notably when certain fields (e.g. range_start) are optional
  let optionalCheckboxStates = $state([] as boolean[][]);

  // modal-related variables
  let showConfirmModal = $state(false);

  let modalLines: {
    fields: {
      removed: string[];
      modified: {
        nameAndType: string;
        kind: string[];
      }[];
      added: string[];
    };
    subtables: {
      removed: string[];
      modified: string[];
      added: string[];
    };
  } = $derived({
    fields: {
      removed: changes.fields.removed.map(
        (f) => `${f.name} (${typeToStr(f.field_kind.type)})`,
      ),

      modified: changes.fields.modified.map((f) => {
        let old = table.old.fields.find(
          (g) => g.field_id === f.field_id,
        ) as Field;

        // get all entries from old field kind
        let oldEntries = Object.entries(old.field_kind);

        // setup comparisons between old and new field to check for changes
        let entries = Object.entries(f.field_kind).map((entry) => {
          return [
            entry[0],
            [
              (oldEntries.find((o) => o[0] === entry[0]) ?? [
                undefined,
                undefined,
              ])[1],
              entry[1],
            ],
          ];
        });

        // add keys from old field that not in new field
        entries.push(
          ...oldEntries
            .filter((o) => entries.findIndex((e) => e[0] === o[0]) === -1)
            .map((o) => [o[0], [o[1], undefined]]),
        );

        return {
          nameAndType:
            f.name !== old.name || f.field_kind.type !== old.field_kind.type
              ? `${old.name} (${typeToStr(old.field_kind.type)}) -> ${f.name} (${typeToStr(f.field_kind.type)})`
              : "",
          kind: entries
            .filter((e) => e[0] !== "type")
            .filter((e) => !recursiveCompare(e[1][0], e[1][1]))
            .filter((e) => !(e[1][0] == null && e[1][1] == null)) // check if both nullish
            .map(
              (e) =>
                `${f.name} [${e[0]}] ${e[1][0] ?? "[Empty]"} -> ${e[1][1] ?? "[Empty]"}`,
            ),
        };
      }),
      added: changes.fields.added.map(
        (f) => `${f.name} (${typeToStr(f.field_kind.type)})`,
      ),
    },
    subtables: {
      removed: changes.subtables.removed.map((t) => t.table.name),
      modified: changes.subtables.modified.map(
        (t) =>
          `${table.old.children.find((u) => u.table.table_id === t.table.table_id)} -> ${t.table.table_id}`,
      ),
      added: changes.subtables.added.map((t) => t.table.name),
    },
  });

  let confirmationModal: HTMLDialogElement;

  //
  // State methods
  //

  /**
   * Add a subtable to table.new
   */
  const addSubtable = (): void => {
    let j = 1;
    let newTableName = "New Table " + j;
    while (table.new.children.some((t) => t.table.name === newTableName)) {
      newTableName = "New Table " + ++j;
    }

    let id = -1;
    while (table.new.children.some((t) => t.table.table_id === id)) {
      id--;
    }

    table.new.children.splice(0, 0, {
      table: {
        table_id: id,
        user_id: -1,
        parent_id: data.table_prop.table_id,
        name: newTableName,
        description: "",
      },
      fields: [],
      entries: [],
      children: [],
    });
  };

  /**
   * Remove a subtable from table.new
   * @param {number} i - the index of the subtable to remove in table.new.children
   */
  const removeSubtable = (i: number): void => {
    table.new.children.splice(i, 1);
  };

  /**
   * Restore a subtable to table.new which is in table.old
   * @param {number} i - the index of the subtable to restore in changes.subtables.removed
   */
  const restoreSubtable = (i: number): void => {
    table.new.children.push($state.snapshot(changes.subtables.removed[i]));
  };

  /**
   * Add a field to table.new
   */
  const addField = (): void => {
    // find unique field name
    let j = 1;
    let newFieldName = "New Field " + j;
    while (table.new.fields.some((f: Field) => f.name === newFieldName)) {
      newFieldName = "New Field " + ++j;
    }

    // find unique field id
    let id = -1;
    while (table.new.fields.some((f) => f.field_id === id)) {
      id--;
    }

    // create new field and add it
    let newField: Field = {
      table_id: table.new.table.table_id,
      user_id: -1,
      field_id: id, // temporary id, will be replaced when created
      ordering: table.new.fields.length,

      name: newFieldName,
      field_kind: {
        type: FieldType.Text,
        is_required: true,
      },
    };

    table.new.fields.push(newField);

    // update optionalCheckBoxStates
    optionalCheckboxStates.push(
      fieldKindInputList[table.new.fields.length - 1].map((v) => !v.optional),
    );

    //clear errors
    table.new.fields.forEach((f) => {
      errors.fields[f.field_id] = "";
    });
  };

  /**
   * Remove a field from table.new
   * @param {number} i - the index of the field to remove in table.new.fields
   */
  const removeField = (i: number): void => {
    table.new.fields.splice(i, 1);
  };

  /**
   * Restore a field to table.new which is in table.old
   * @param {number} i - the index of the feild to restore in changes.fields.removed
   */
  const restoreField = (i: number): void => {
    table.new.fields.push($state.snapshot(changes.fields.removed[i]));
  };

  /**
   * Update optionalCheckboxStates
   */
  const updateAllOptionalCheckboxes = () => {
    fieldKindInputList.forEach((val, i) => {
      updateOptionalCheckbox(i);
    });
  };

  /**
   * Update the state of a field's checkboxes in optionalCheckboxStates
   * @param {number} i - the index of the field in table.new.fields
   */
  const updateOptionalCheckbox = (i: number) => {
    optionalCheckboxStates[i] = fieldKindInputList[i].map(
      (v) =>
        !v.optional ||
        ((table.new.fields[i].field_kind as any)[v.name] !== null &&
          (table.new.fields[i].field_kind as any)[v.name] !== undefined),
    );
  };

  /**
   * Open the modal
   */
  const openConfirmationModal = () => {
    confirmationModal?.showModal();
  };

  /**
   * Close the modal
   */
  const closeConfirmationModal = () => {
    confirmationModal?.close();
  };

  //
  // Helper methods
  //

  /**
   * Generate FieldKindInputParameters for the is_required FieldKind value for a Field
   * @param {number} i - The index of the field in table.new.fields associated with the input
   * @returns {FieldKindInputParameters}
   */
  const getRequiredFieldKindInput = (i: number): FieldKindInputParameters => {
    return {
      name: "is_required",
      label: "Is Required",
      type: "checkbox",
      optional: false,
      bindGetter: () => {
        return (table.new.fields[i].field_kind as RequirableKind).is_required;
      },
      bindSetter: (val: boolean) => {
        (table.new.fields[i].field_kind as RequirableKind).is_required = val;
      },
    };
  };

  /**
   * Generate FieldKindInputParameters for the type FieldKind value for a Field
   * @param {number} i - The index of the field in table.new.fields associated with the input
   * @returns {FieldKindInputParameters}
   */
  const getTypeFieldKindInput = (i: number): FieldKindInputParameters => {
    return {
      name: "type",
      label: "Type",
      type: "select",
      optional: false,
      selectOptions: Object.fromEntries(
        fieldTypes.map((t) => [t, typeToStr(t)]),
      ),
      bindGetter: () => {
        return table.new.fields[i].field_kind.type;
      },
      bindSetter: (val: FieldType) => {
        // swap out field option if type change
        if (val != table.new.fields[i].field_kind.type) {
          switch (val) {
            case FieldType.Text:
              table.new.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Integer:
              table.new.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Decimal:
              table.new.fields[i].field_kind = {
                type: val,
                is_required: true,

                scientific_notation: true,
              };
              break;
            case FieldType.Money:
              table.new.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Progress:
              table.new.fields[i].field_kind = {
                type: val,
                total_steps: 100,
              };
              break;
            case FieldType.DateTime:
              table.new.fields[i].field_kind = {
                type: val,
                is_required: true,

                date_time_format: "YYYY-MM-DD",
              };
              break;
            case FieldType.WebLink:
              table.new.fields[i].field_kind = {
                type: val,
                is_required: true,
              };
              break;
            case FieldType.Checkbox:
              table.new.fields[i].field_kind = {
                type: val,
              };
              break;
            case FieldType.Enumeration:
              table.new.fields[i].field_kind = {
                type: val,
                is_required: true,
                values: {} as { [key: number]: string },
                default_value: 0,
              };
              break;
          }
          updateOptionalCheckbox(i);
        }
      },
    };
  };

  /**
   * Recursively compare two Javascript values
   * @param {any} a - The first value
   * @param {any} b - The second value
   * @returns {boolean} - whether the two values are equivalent
   */
  const recursiveCompare = (a: any, b: any): boolean => {
    if (typeof a !== typeof b) return false;
    if (a === null || b === null) {
      return a === null && b === null;
    } else if (Array.isArray(a)) {
      // compare every element
      return a.every((obj, i) => recursiveCompare(obj, b[i]));
    } else if (a instanceof Date) {
      // compare time since epoch
      return b instanceof Date && a.getTime() === b.getTime();
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

  //
  // API Calls
  //

  const deleteCurrentTable = async () => {
    await deleteTable(data.table_prop);
    await invalidateAll();
    goto("/tables");
  };

  const loadFields = () => {
    getTableData(data.table_prop.table_id.toString()).then((td) => {
      // update fields
      table.old.fields = td.fields.toSorted((f, g) => f.ordering - g.ordering);
      table.new.fields = $state.snapshot(table.old.fields);

      //update optionalCheckboxStates
      updateAllOptionalCheckboxes();
      table.new.fields.forEach((f) => {
        errors.fields[f.field_id] = "";
      });

      // update subtables
      table.old.children = td.children;
      table.new.children = $state.snapshot(table.old.children);

      table.new.children.forEach((subtable) => {
        errors.subtables[subtable.table.table_id] = "";
      });
    });
  };

  // unifies POST, PUT and DELETE methods into one method to be run when the user confirms the modifications
  // TODO: Fix race condition so that ordering is guaranteed
  const saveFields = () => {
    let promises = [];

    showConfirmModal = false;

    // modify table name/description
    if (
      table.new.table.name !== table.old.table.name ||
      table.new.table.description !== table.old.table.description
    )
      promises.push(
        patchTable(table.new.table)
          .then((response: Table) => {
            table.old.table.name = response.name;
            table.old.table.description = response.description;
            errors.metadata = "";
            return { ok: true };
          })
          .catch((e: APIError) => {
            errors.metadata = e.body.toString();
            return { ok: false };
          }),
      );

    // create new fields
    changes.fields.added.forEach((field) => {
      promises.push(
        postField(field)
          .then((response: Field) => {
            let newField = response;
            table.old.fields.push(newField);
            table.new.fields[
              table.new.fields.findIndex((f) => f.field_id === field.field_id)
            ].field_id = newField.field_id;
            errors.fields[field.field_id] = "";
            return { ok: true };
          })
          .catch((e: APIError) => {
            let text = e.body.toString();
            errors.fields[field.field_id] = text;

            return { ok: false };
          }),
      );
    });

    // modify existing fields
    changes.fields.modified.forEach((field) => {
      promises.push(
        patchField(field)
          .then((response: Field) => {
            table.old.fields[
              table.old.fields.findIndex((f) => f.field_id === field.field_id)
            ] = response;
            errors.fields[field.field_id] = "";
            return { ok: true };
          })
          .catch((e: APIError) => {
            let text = e.body.toString();
            errors.fields[field.field_id] = text;
            return { ok: false };
          }),
      );
    });

    // delete fields
    for (const field of changes.fields.removed) {
      promises.push(
        deleteField(field)
          .then(() => {
            table.old.fields.splice(
              table.old.fields.findIndex((f) => f.field_id === field.field_id),
              1,
            );
            return { ok: true };
          })
          .catch(() => {
            errors.fields[field.field_id] = "Could not delete";
            return { ok: false };
          }),
      );
    }

    // add subtables
    changes.subtables.added.forEach((t) => {
      promises.push(
        postCreateTable(t.table)
          .then((response: Table) => {
            let newTableData = {
              table: response,
              fields: [],
              entries: [],
              children: [],
            };
            table.old.children.splice(0, 0, newTableData);
            table.new.children[
              table.old.children.findIndex(
                (u) => u.table.table_id === t.table.table_id,
              )
            ] = newTableData;
            errors.subtables[t.table.table_id] = "";
            return { ok: true };
          })
          .catch((e) => {
            errors.subtables[t.table.table_id] = e.body.toString();
            return { ok: false };
          }),
      );
    });

    // modify subtables
    changes.subtables.modified.forEach((t) => {
      promises.push(
        patchTable(t.table)
          .then((response: Table) => {
            let modifiedTableData = {
              table: response,
              fields: [],
              entries: [],
              children: [],
            };

            table.old.children[
              table.old.children.findIndex(
                (u) => u.table.table_id === t.table.table_id,
              )
            ] = modifiedTableData;
            errors.subtables[t.table.table_id] = "";
            return { ok: true };
          })
          .catch((e) => {
            errors.subtables[t.table.table_id] = e.body.toString();
            return { ok: false };
          }),
      );
    });

    // delete subtables
    changes.subtables.removed.forEach((t) => {
      promises.push(
        deleteTable(t.table)
          .then(() => {
            table.old.children.splice(
              table.old.children.findIndex(
                (u) => u.table.table_id === t.table.table_id,
              ),
              1,
            );
            return { ok: true };
          })
          .catch(() => {
            errors.subtables[t.table.table_id] = "Could not delete";
            return { ok: false };
          }),
      );
    });

    // quit or reload
    Promise.allSettled(promises).then((results) => {
      if (results.every((r) => r.status == "fulfilled" && r.value.ok)) {
        goto(`/tables/${data.table_prop.parent_id}`);
      }
    });
  };

  //
  // Startup
  //

  onMount(() => {
    loadFields();

    updateAllOptionalCheckboxes();

    table.new.fields.forEach((f) => {
      errors.fields[f.field_id] = "";
    });
  });
</script>

<div class="w-full">
  <div class="mb-4">
    <!-- Top bar -->
    <label class="input">
      Name:
      <input
        id="name-input"
        bind:value={table.new.table.name}
        disabled={isSubtable}
      />
    </label>
    <label class="input">
      Description:
      <input
        id="description-input"
        bind:value={table.new.table.description}
        disabled={isSubtable}
      />
    </label>
    {#if errors.metadata !== ""}
      <p class="text-error">{errors.metadata}</p>
    {/if}
    {#if !isSubtable}
      <ConfirmButton
        initClass=""
        confirmClass="btn-error"
        class="btn"
        initText="Delete Table"
        confirmText="Confirm Delete"
        onconfirm={deleteCurrentTable}
      />
    {/if}
  </div>
  <!-- Fields  -->
  <div
    class="flex justify-between w-full flex-nowrap overflow-scroll gap-3 py-4"
  >
    <div class="flex items-stretch gap-3">
      <!-- Field editing sections -->
      {#each table.new.fields as field, i}
        <div
          class="card min-w-64 bg-base-100 p-4 shadow-md flex flex-col gap-3 justify-between"
        >
          <div class="flex flex-col gap-3">
            <!-- Field name -->
            <input class="input w-full" bind:value={table.new.fields[i].name} />

            <!-- Field kind parameters -->
            {#each fieldKindInputList[i] as fieldKindInput, j}
              <div class="my-2">
                <div class="flex items-center">
                  <!-- Add checkbox to enable/disable input if it is optional -->
                  {#if fieldKindInput.optional}
                    <input
                      class="checkbox mr-3"
                      type="checkbox"
                      bind:checked={
                        () => optionalCheckboxStates[i][j],
                        (val) => {
                          optionalCheckboxStates[i][j] = val;
                          if (val) {
                            (table.new.fields[i].field_kind as any)[
                              fieldKindInput.name
                            ] = fieldKindInput.default;
                          } else {
                            delete (table.new.fields[i].field_kind as any)[
                              fieldKindInput.name
                            ];
                          }
                        }
                      }
                    />
                  {/if}
                  <!-- The input -->
                  <VariableInput
                    class={["textarea", "select", "checkbox"].includes(
                      fieldKindInput.type,
                    )
                      ? fieldKindInput.type
                      : "input"}
                    params={fieldKindInput}
                    disabled={!optionalCheckboxStates[i][j]}
                    id={fieldKindInput.label + i}
                  />
                </div>
              </div>
            {/each}
          </div>
          <button onclick={() => removeField(i)} class="btn btn-error w-full"
            >Remove</button
          >
          <!-- Error -->
          {#if errors.fields[field.field_id] !== ""}
            <div class="rounded-lg text-error">
              {errors.fields[field.field_id]}
            </div>
          {/if}
        </div>
      {/each}

      <!-- Deleted but restorable fields -->
      {#each changes.fields.removed as field, i}
        <button
          class="btn btn-dash btn-error hover:btn-success border-2 h-72 w-64 flex flex-col"
          onclick={() => restoreField(i)}
        >
          <p class="font-bold block">
            {field.name} ({typeToStr(field.field_kind.type)})
          </p>
          <p class="font-bold block">Click to Restore</p>
        </button>
      {/each}

      <!-- Add field button -->
      <button
        class="btn btn-dash border-2 min-h-72 h-full w-64"
        onclick={addField}
        aria-label="add field">Add Field</button
      >
    </div>

    <!-- Subtables -->
    {#if !isSubtable}
      <div class="flex justify-end gap-3">
        <!-- Add subtable button -->
        <button
          class="btn btn-dash border-2 min-h-72 h-full w-64"
          onclick={addSubtable}
          aria-label="add Subtable">Add Subtable</button
        >

        <!-- Subtable sections -->
        {#each table.new.children as subtable, i}
          <div
            class="card w-64 bg-base-100 p-4 shadow-md flex flex-col justify-between gap-3"
          >
            <input
              class="input"
              bind:value={table.new.children[i].table.name}
            />
            <button
              onclick={() => removeSubtable(i)}
              class="btn btn-error w-full">Remove</button
            >

            {#if errors.subtables[subtable.table.table_id]}
              <p class="text-error">
                {errors.subtables[subtable.table.table_id]}
              </p>
            {/if}
          </div>
        {/each}
        {#each changes.subtables.removed as subtable, i}
          <button
            class="btn btn-dash btn-error hover:btn-success border-2 h-72 w-64 flex flex-col"
            onclick={() => restoreSubtable(i)}
          >
            <p class="font-bold">
              {subtable.table.name}
            </p>
            <p class="font-bold block">Click to Restore</p>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Bottom Bar -->
{#if table.old !== table.new}
  <!-- TODO: actually have the condition check for modifications -->
  <div class="flex justify-center gap-4">
    <button onclick={openConfirmationModal} class="btn join-item">Save</button>
    <button
      onclick={() => goto(`/tables/${data.table_prop.table_id}`)}
      class="btn btn-soft btn-error join-item">Cancel</button
    >
  </div>
{/if}

<!-- Confirmation modal -->
<dialog class="modal" bind:this={confirmationModal}>
  <div class="modal-box">
    <h2 class="w-full font-bold text-center">Edit Summary</h2>
    <!-- Table name + description -->
    {#if table.new.table.name !== table.old.table.name}
      <p>
        <span class="font-bold">Changed Title:</span> "{table.old.table.name}"
        -&gt "{table.new.table.name}"
      </p>
    {/if}
    {#if table.new.table.description !== table.old.table.description}
      <p>
        <span class="font-bold">Changed Description:</span> "{table.old.table
          .description}" -&gt "{table.new.table.description}"
      </p>
    {/if}

    <!-- Added fields -->
    {#each modalLines.fields.added as line}
      <p><span class="font-bold text-success">Added Field:</span> {line}</p>
    {/each}

    <!-- Modified fields -->
    {#each modalLines.fields.modified as moddedField}
      {#if moddedField.nameAndType}
        <p>
          <span class="font-bold text-warning">Change Field:</span>
          {moddedField.nameAndType}
        </p>
      {/if}
      {#each moddedField.kind as line}
        <p><span class="font-bold">Change Field Property:</span> {line}</p>
      {/each}
    {/each}

    <!-- Deleted fields -->
    {#each modalLines.fields.removed as line}
      <p>
        <span class="font-bold text-error">[!]</span>
        <span class="font-bold">Delete Field:</span>
        {line}
      </p>
    {/each}

    <!-- Added subtables -->
    {#each modalLines.subtables.added as line}
      <p><span class="font-bold">Added Subtable:</span> {line}</p>
    {/each}

    <!-- Modified subtables -->
    {#each modalLines.subtables.modified as line}
      <p><span class="font-bold">Change Subtable:</span> {line}</p>
    {/each}

    <!-- Deleted subtables -->
    {#each modalLines.subtables.removed as line}
      <p>
        <span class="font-bold text-error">[!]</span>
        <span class="font-bold">Delete Field:</span>
        {line}
      </p>
    {/each}

    <!-- Button cluster -->
    <div class="flex justify-center items-center gap-2 mt-2">
      <button class="btn" onclick={saveFields}>Confirm</button>
      <button class="btn btn-soft btn-error" onclick={closeConfirmationModal}
        >Cancel</button
      >
    </div>
  </div>
</dialog>

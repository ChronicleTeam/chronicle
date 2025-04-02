// Field
export enum FieldType {
  Text = "Text",
  Integer = "Integer",
  Decimal = "Float",
  Money = "Money",
  Progress = "Progress",
  DateTime = "DateTime",
  WebLink = "WebLink",
  Checkbox = "Checkbox",
  Enumeration = "Enumeration",
}
export type Field = {
  table_id: number;
  user_id: number;
  field_id: number;
  name: string;
  ordering: number;
  field_kind: FieldKind;
  updated_at?: Date;
};

export type TextKind = {
  type: FieldType.Text;
  is_required: boolean;
};

export type IntegerKind = {
  type: FieldType.Integer;
  is_required: boolean;
  range_start?: number;
  range_end?: number;
};

export type DecimalKind = {
  type: FieldType.Decimal;
  is_required: boolean;
  range_start?: number;
  range_end?: number;
  scientific_notation: boolean;
  number_precision?: number;
  number_scale?: number;
};

export type MoneyKind = {
  type: FieldType.Money;
  is_required: boolean;
  range_start?: string;
  range_end?: string;
};

export type ProgressKind = {
  type: FieldType.Progress;
  total_steps: number;
};

export type DateTimeKind = {
  type: FieldType.DateTime;
  is_required: boolean;
  range_start?: Date;
  range_end?: Date;
  date_time_format: string;
};

export type WebLinkKind = {
  type: FieldType.WebLink;
  is_required: boolean;
}

export type CheckboxKind = {
  type: FieldType.Checkbox;
}
export type EnumerationKind = {
  type: FieldType.Enumeration;
  is_required: boolean;
  values: { [key: number]: string };
  default_value: number;
};

export type FieldKind =
  | TextKind
  | IntegerKind
  | DecimalKind
  | MoneyKind
  | ProgressKind
  | DateTimeKind
  | WebLinkKind
  | CheckboxKind
  | EnumerationKind

export const typeToStr = (t: FieldType): string => {
  switch (t) {
    case FieldType.Decimal:
      return "Decimal";
    case FieldType.DateTime:
      return "Date Time";
    default:
      return t;
  }
}

// Data table
export type Table = {
  table_id: number;
  user_id: number;
  parent_id?: number | null;
  name: string;
  description: string;
  created_at?: Date;
  updated_at?: Date;
};
export type TableData = {
  table: Table;
  fields: Field[];
  entries: Entry[];
  children: TableData[];
};

// Entry
export type Entry = {
  entry_id: number;
  parent_id?: number | null;
  cells: Cells;
};



// Cell
export type Cells = {
  [key: string]: Cell;
}


export type Text = string;
export type Integer = number;
export type Decimal = number;
export type Money = string;
export type Progress = number;
export type DateTime = Date;
export type Weblink = string;
export type Checkbox = boolean;
export type Enumeration = number;

export type Cell =
  | Text
  | Integer
  | Decimal
  | Money
  | Progress
  | DateTime
  | Weblink
  | Checkbox
  | Enumeration



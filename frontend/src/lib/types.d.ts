// Field
export enum FieldType {
  Text = "Text",
  Integer = "Integer",
  Decimal = "Decimal",
  Money = "Money",
  Progress = "Progress",
  DateTime = "DateTime",
  Interval = "Interval",
  WebLink = "WebLink",
  Email = "Email",
  Checkbox = "Checkbox",
  Enumeration = "Enumeration",
  Image = "Image",
  File = "File",
}
export type Field = {
  table_id: number;
  user_id: number;
  field_id: number;
  name: string;
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

export type IntervalKind = {
  type: FieldType.Interval;
  is_required: boolean;
}

export type WebLinkKind = {
  type: FieldType.WebLink;
  is_required: boolean;
}

export type EmailKind = {
  type: FieldType.Email;
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

export type ImageKind = {
  type: FieldType.Image;
  is_required: boolean;
}
export type FileKind = {
  type: FieldType.File;
  is_required: boolean;
}

export type FieldKind =
  | TextKind
  | IntegerKind
  | DecimalKind
  | MoneyKind
  | ProgressKind
  | DateTimeKind
  | IntervalKind
  | WebLinkKind
  | EmailKind
  | CheckboxKind
  | EnumerationKind
  | ImageKind
  | FileKind;

// Data table
export type Table = {
  table_id: Id;
  user_id: Id;
  name: string;
  description: string;
  created_at: Date;
  updated_at?: Date;
};
export type DataTable = {
  table: Table;
  fields: Field[];
  entries: Entry[];
};

// Entry
export type Entry = {
  entry_id: number;
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
export type Interval = null;
export type Weblink = string;
export type Email = string;
export type Checkbox = boolean;
export type Enumeration = number;
export type Image = null;
export type File = null;

export type Cell =
  | Text
  | Integer
  | Decimal
  | Money
  | Progress
  | DateTime
  | Interval
  | Weblink
  | Email
  | Checkbox
  | Enumeration
  | Image
  | File;

// Variable Inputs

export type InputType =
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

export type InputParameters =
  | {
    label: string;
    type: InputType;
    bindSetter: (val: any) => void;
    bindGetter: () => string | boolean | number;
    min?: undefined;
    max?: undefined;
    step?: undefined;
  }
  | {
    label: string;
    type: "number" | "range";
    bindSetter: (val: any) => void;
    bindGetter: () => string | boolean | number;
    min?: number;
    max?: number;
    step?: number;
  }
  | {
    label: string;
    type: "date" | "datetime-local";
    bindSetter: (val: any) => void;
    bindGetter: () => string | boolean | number;
    min?: Date;
    max?: Date;
    step?: Date;
  }
  | {
    label: string;
    type: "select";
    selectOptions: string[];
    bindSetter: (val: any) => void;
    bindGetter: () => string | boolean | number;
    min?: undefined;
    max?: undefined;
    step?: undefined;
  }
  | {
    label: string;
    type: "checkbox";
    bindSetter: (val: any) => void;
    bindGetter: () => boolean;
    min?: undefined;
    max?: undefined;
    step?: undefined;
  }
  | {
    label: string;
    type: "textarea";
    bindSetter: (val: string) => void,
    bindGetter: () => string
    min?: undefined;
    max?: undefined;
    step?: undefined;
  };


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
  name: string;
  options: FieldOptions;
  updated_at?: Date;
};

export type TextOptions = {
  type: FieldType.Text;
  is_required: boolean;
};

export type IntegerOptions = {
  type: FieldType.Integer;
  is_required: boolean;
  range_start?: number;
  range_end?: number;
};

export type DecimalOptions = {
  type: FieldType.Decimal;
  is_required: boolean;
  range_start?: number;
  range_end?: number;
  scientific_notation: boolean;
  number_precision?: number;
  number_scale?: number;
};

export type MoneyOptions = {
  type: FieldType.Money;
  is_required: boolean;
  range_start?: Decimal;
  range_end?: Decimal;
};

export type ProgressOptions = {
  type: FieldType.Progress;
  total_steps: number;
};

export type DateTimeOptions = {
  type: FieldType.DateTime;
  is_required: boolean;
  range_start?: Date;
  range_end?: Date;
  date_time_format: string;
};

export type IntervalOptions = {
  type: FieldType.Interval;
  is_required: boolean;
}

export type WebLinkOptions = {
  type: FieldType.WebLink;
  is_required: boolean;
}

export type EmailOptions = {
  type: FieldType.Email;
  is_required: boolean;
}

export type CheckboxOptions = {
  type: FieldType.Checkbox;
}
export type EnumerationOptions = {
  type: FieldType.Enumeration;
  is_required: boolean;
  values: Map;
  default: number;
};

export type ImageOptions = {
  type: FieldType.Image;
  is_required: boolean;
}
export type FileOptions = {
  type: FieldType.File;
  is_required: boolean;
}

export type FieldOptions =
  | TextOptions
  | IntegerOptions
  | DecimalOptions
  | MoneyOptions
  | ProgressOptions
  | DateTimeOptions
  | IntervalOptions
  | WebLinkOptions
  | EmailOptions
  | CheckboxOptions
  | EnumerationOptions
  | ImageOptions
  | FileOptions;

// Data table
export type DataTable = {
  table: Table;
  fields: Field[];
  entries: Entry[];
};

// Entry
export type Entry = {
  cells: Cell[];
};

// Cell
export type Text = string;
export type Integer = number;
export type Decimal = number;
export type Money = Decimal;
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

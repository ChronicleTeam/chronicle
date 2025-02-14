
// Field
export type Field = {
  name: string;
  options: FieldOptions;
  updated_at?: Date;
};

type baseOptions = {
  type: string;
};

type reqBaseOptions = baseOptions & {
  is_required: boolean;
}

export type TextOptions = reqBaseOptions;

export type IntegerOptions = reqBaseOptions & {
  range_start?: number;
  range_end?: number;
};

export type DecimalOptions = reqBaseOptions & {
  scientific_notation: boolean;
  number_precision?: number;
  number_scale?: number;
};

export type MoneyOptions = reqBaseOptions & {
  range_start?: Decimal;
  range_end?: Decimal;
};

export type ProgressOptions = baseOptions & {
  total_steps: number;
};

export type DateTimeOptions = reqBaseOptions & {
  range_start?: Date;
  range_end?: Date;
  date_time_format: string;
};

export type IntervalOptions = reqBaseOptions;

export type WebLinkOptions = reqBaseOptions;

export type EmailOptions = reqBaseOptions;

export type CheckboxOptions = baseOptions;

export type EnumerationOptions = reqBaseOptions & {
  values: Map;
  default: number;
}

export type ImageOptions = reqBaseOptions;
export type FileOptions = reqBaseOptions;

export type FieldOptions = TextOptions | IntegerOptions | DecimalOptions | MoneyOptions | ProgressOptions | DateTimeOptions | IntervalOptions | WebLinkOptions | EmailOptions | CheckboxOptions | EnumerationOptions | ImageOptions | FileOptions;

// Data table
export type DataTable = {
  table: Table;
  fields: Field[];
  entries: Entry[];
};

// Entry
export type Entry = {
  cells: Cell[] 
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
export type Checkbox = boolean
export type Enumeration = number;
export type Image = null;
export type File = null;

export type Cell = Text | Integer | Decimal | Money | Progress | DateTime | Interval | Weblink | Email | Checkbox | Enumeration | Image | File;

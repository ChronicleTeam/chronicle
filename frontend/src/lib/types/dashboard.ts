import type { FieldKind, Cells } from "./dataManagement.js";
// Charts

export type Dashboard = {
  dashboard_id: number;
  user_id: number;
  name: string;
  description: string;
}

export type Chart = {
  chart_id: number;
  dashboard_id: number;
  table_id: number;
  name: string;
  chart_kind: ChartKind;
}

export enum ChartKind {
  Table = "Table",
  Bar = "Bar",
  Line = "Line"
}

export type Axis = {
  axis_id: number;
  chart_id: number;
  field_id: number;
  axis_kind: AxisKind;
  aggregate?: Aggregate;
}

export enum AxisKind {
  X = "X",
  Y = "Y",
  Color = "Color",
  Size = "Size",
  Tooltip = "Tooltip",
  Label = "Label",
  Detail = "Detail",
}

export enum Aggregate {
  Sum = "Sum",
  Average = "Average",
  Min = "Min",
  Max = "Max",
  Count = "Count",
}

export type AxisField = {
  axis: Axis;
  field_name: string;
  field_kind: FieldKind;
}

export type ChartData = {
  chart: Chart;
  axes: AxisField[];
  cells: Cells[];
}



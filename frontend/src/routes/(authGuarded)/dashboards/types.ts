import type { AxisField, TableData, Chart } from "$lib/types";

export enum EditMode {
  DISPLAY = 0,
  EDIT_DASH = 1,
  EDIT_CHART = 2
}

export type ModeState = {
  mode: EditMode.DISPLAY;
} | {
  mode: EditMode.EDIT_DASH;
  metadataChanged: boolean;
  newChart: Chart | null;
} | {
  mode: EditMode.EDIT_CHART;
  chartIdx: number;
  chartTableData: TableData;
  axisFields: AxisField[];

}

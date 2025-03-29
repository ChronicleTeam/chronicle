import type { TableData } from "$lib/types.d.js";
export enum TableMode {
  DISPLAY = 0,
  INSERT = 1,
  EDIT = 2,
  CHILD = 3,
  EDIT_CHILD = 4,
};

export type TableChild = {

  table_data: TableData;
  entry_id: number;
}


export type ModeState = {
  mode: TableMode.DISPLAY
} | {
  mode: TableMode.INSERT;
  entry_idx: number;
} | {
  mode: TableMode.EDIT;
  entry_idx: number;
} | {
  mode: TableMode.CHILD;
  child: TableChild;
} | {
  mode: TableMode.EDIT_CHILD;
  child: TableChild;
}

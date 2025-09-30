import type { TableData } from "$lib/types";
export enum TableMode {
  DISPLAY = 0,
  INSERT = 1,
  EDIT = 2,
};

export type TableChild = {

  table_data: TableData;
  entry_id: number;
}


export type ModeState = {
  mode: TableMode.DISPLAY
} | {
  mode: TableMode.INSERT;
  entry_idxes: number[];
} | {
  mode: TableMode.EDIT;
  entry_idx: number;
} 

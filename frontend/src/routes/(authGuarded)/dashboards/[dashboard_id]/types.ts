import type { Chart } from "$lib/types";

export enum EditMode {
  DISPLAY = 0,
  EDIT_DASH = 1,
}

export type ModeState = {
  mode: EditMode.DISPLAY;
} | {
  mode: EditMode.EDIT_DASH;
  metadataChanged: boolean;
  newChart: Chart | null;
} 

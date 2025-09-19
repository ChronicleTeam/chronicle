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
    selectOptions: string[] | { [key: string | number]: string };
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

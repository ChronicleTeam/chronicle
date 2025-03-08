<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  let {
    initText,
    confirmText = "Confirm",
    onconfirm,
    initClass = "bg-white hover:bg-gray-100",
    confirmClass = "bg-red-400 hover:bg-red-500 ",
    class: btnClass = "rounded-md px-2 py-1 transition",
  }: {
    initText: string;
    confirmText?: string;
    onconfirm: (e: MouseEvent) => void;
    initClass?: string;
    confirmClass?: string;
    class?: ClassValue;
  } = $props();

  let clicked = $state(false);
</script>

<button
  class={[btnClass, clicked ? confirmClass : initClass]}
  onclick={clicked
    ? onconfirm
    : () => {
        clicked = true;
      }}
  onfocusout={() => {
    clicked = false;
  }}
>
  {clicked ? confirmText : initText}
</button>

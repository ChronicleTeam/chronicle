<script lang="ts">
  import type { ClassValue } from "svelte/elements";

  let {
    initText, // text on the button before first click
    confirmText = "Confirm", // text on the button after first click
    onconfirm, // function to run after the user clicks a second time
    initClass = "btn", // classes on the button ONLY before first click
    confirmClass = "btn-error", // classes on the button ONLY after the first click
    class: btnClass = "btn", // classes on the button which are ALWAYS active
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

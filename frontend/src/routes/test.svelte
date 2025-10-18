<script lang="ts">
  import { goto } from "$app/navigation";
  function goHome() {
    goto("/");
  }
</script>

<button on:click={goHome}>Home</button>

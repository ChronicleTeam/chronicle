<script lang="ts">
  import { goto } from "$app/navigation";
  import { logout } from "$lib/user.svelte";
  import { onMount } from "svelte";

  let message = $state("Logging out...");

  onMount(() => {
    logout().then(() => {
      message = "Logged out.";
      goto("/");
    });
  });
</script>

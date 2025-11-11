<script lang="ts">
  import type { PageProps } from "./$types";
  import TableEditor from "$lib/components/dataManagement/TableEditor.svelte";
  import { user } from "$lib/user.svelte";

  //
  // State
  //

  // the TableData object being displayed
  let { data }: PageProps = $props();
  let table = $state(data.table);
  $effect(() => {
    table = data.table;
  });

  let allUsers = $derived(data.users);
  let userAccess = $derived(data.allAccess);
  let accessRole = $derived(data.role);
</script>

{#await user() then u}
  {#if u}
    <TableEditor {table} {allUsers} {userAccess} {accessRole} user={u} />
  {/if}
{/await}

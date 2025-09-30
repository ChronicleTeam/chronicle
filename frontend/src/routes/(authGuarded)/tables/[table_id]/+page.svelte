<script lang="ts">
  import type { PageProps } from "./$types";
  import TableEditor from "$lib/components/dataManagement/TableEditor.svelte";

  //
  // State
  //

  // the TableData object being displayed
  let { data }: PageProps = $props();
  let table = $state(data.table);
  $effect(() => {
    table = data.table;
  });
</script>

<TableEditor {table} />

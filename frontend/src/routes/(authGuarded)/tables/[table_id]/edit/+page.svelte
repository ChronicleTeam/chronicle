<script lang="ts">
  import type { PageProps } from "./$types";
  import FieldEditor from "$lib/components/dataManagement/FieldEditor.svelte";

  let { data }: PageProps = $props();
</script>

<FieldEditor loadedData={data} />

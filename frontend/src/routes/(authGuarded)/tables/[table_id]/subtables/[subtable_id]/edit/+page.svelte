<script lang="ts">
  import FieldEditor from "$lib/components/dataManagement/FieldEditor.svelte";

  let { data } = $props();
</script>

<FieldEditor loadedData={data} />

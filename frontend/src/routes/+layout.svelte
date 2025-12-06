<script lang="ts">
  import "../app.css";
  let { children } = $props();
</script>

<!-- just sets the outer div to be the full screen height -->
<div class="h-screen">
  {@render children()}
</div>

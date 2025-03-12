<script lang="ts">
  import "../app.css";
  let { children } = $props();

  type navLink = {
    page: string;
    text: string;
  };

  const links: navLink[] = [
    {
      page: "/tables",
      text: "Data Management",
    },
    {
      page: "/dashboards",
      text: "Dashboards",
    },
  ];
</script>

<div class="p-4">
  <!-- Navbar -->
  <nav class="flex bg-gray-200 mb-4 rounded-lg">
    {#each links as link}
      <a
        class="px-3 py-2 bg-gray-200 hover:bg-white transition rounded-lg font-bold"
        href={link.page}>{link.text}</a
      >
    {/each}
  </nav>
  {@render children()}
</div>

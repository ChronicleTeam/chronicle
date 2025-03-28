<script lang="ts">
  import { user } from "$lib/user.svelte";

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
    {
      page: "/logout",
      text: "Logout",
    },
  ];
</script>

<div class="p-4 h-screen">
  {#await user() then u}
    {#if u}
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
    {:else}
      <p>Not authorized.</p>
      <a href="/">Go home</a>
    {/if}
  {/await}
</div>

<script lang="ts">
  //import { user } from "$lib/user.svelte";
  const user = async () => "hello";

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

<div class="p-4 h-full flex flex-col">
  {#await user() then u}
    {#if u}
      <!-- Navbar -->
      <nav class="navbar bg-base-300 mb-4 rounded-lg shadow-xs">
        <div class="navbar-start">
          <div class="dropdown">
            <div tabindex="0" role="button" class="btn btn-square btn-ghost">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="m-1 h-6 w-6 stroke-base-content"
                fill="none"
                viewBox="0 0 24 24"
              >
                <line x1="0" y1="6" x2="24" y2="6" />
                <line x1="0" y1="12" x2="24" y2="12" />
                <line x1="0" y1="18" x2="24" y2="18" />
              </svg>
            </div>
            <ul
              tabindex="0"
              class="menu menu-sm dropdown-content bg-base-100 z-1 w-52 p-2 shadow-xs rounded-xs"
            >
              {#each links as link}
                <li>
                  <a href={link.page}>{link.text}</a>
                </li>
              {/each}
            </ul>
          </div>
        </div>
        <div class="navbar-center">
          <h1 class="text-base-content text-4xl font-bold">Chronicle</h1>
        </div>
      </nav>
      {@render children()}
    {:else}
      <p>Not authorized.</p>
      <a href="/">Go home</a>
    {/if}
  {/await}
</div>

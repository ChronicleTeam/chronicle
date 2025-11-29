<script lang="ts">
  import { postDashboard, type APIError } from "$lib/api";
  import { page } from "$app/state";
  import { invalidateAll } from "$app/navigation";

  let { children, data } = $props();
  let dashboards = $derived(
    data.dashboards.map((dashboardItem) => dashboardItem.dashboard),
  );
  $inspect(dashboards);

  //
  // State
  //

  // error fields
  let errors = $state({
    dashboard: {
      add: "",
      remove: "",
    },
  });

  // variables for dashboard editor
  let addDashMode = $state(false);
  let addDashName = $state("");

  //
  // API calls
  //

  const addDashboard = (name: string) =>
    postDashboard(name)
      .then(() => {
        addDashMode = false;
        addDashName = "";
        errors.dashboard.add = "";
        invalidateAll();
      })
      .catch((e: APIError) => {
        errors.dashboard.add =
          "Error: " + (e.body as { [key: string]: string }).name;
      });
</script>

<div class="flex flex-wrap gap-4 size-full items-stretch">
  <!-- Sidebar -->
  <div class="basis-48 grow bg-base-300 rounded-lg shadow-xs">
    <!-- Dashboard list -->
    <ul class="menu w-full">
      <li class="menu-title">Dashboards</li>
      {#each dashboards as d}
        <li>
          <a
            class={{
              "menu-active":
                d.dashboard_id.toString() === page.params.dashboard_id,
            }}
            href={`/dashboards/${d.dashboard_id}`}>{d.name}</a
          >
        </li>
      {/each}
    </ul>
    <!-- Dashboard creation input -->
    <div
      class="collapse collapse-plus bg-base-100 stroke-base-200 rounded-md mx-2 w-auto"
    >
      <input type="checkbox" />
      <div class="collapse-title text-sm">Add Dashboard</div>
      <div class="collapse-content flex flex-col">
        <p class="font-semibold mb-4">New Dashboard</p>
        <div class="join">
          <input
            class="input join-item"
            bind:value={addDashName}
            id="table-name-input"
          />
          <button
            onclick={() => addDashboard(addDashName)}
            class="btn join-item">Create</button
          >
        </div>

        {#if errors.dashboard.add !== ""}
          <p class="text-error">{errors.dashboard.add}</p>
        {/if}
      </div>
    </div>
  </div>
  <!-- Main editor -->
  <div
    class="bg-base-300 shadow-xs basis-xl grow-5 shrink min-w-0 rounded-lg p-3"
  >
    {@render children()}
  </div>
</div>

<script lang="ts">
  import { type Dashboard } from "$lib/types";
  import {
    deleteDashboard,
    getDashboards,
    postDashboard,
    type APIError,
  } from "$lib/api";
  import DashboardEditor from "./DashboardEditor.svelte";

  //
  // State
  //

  // promise which fetches the dashboards
  let asyncDashboards: Promise<Dashboard[]> = $state(getDashboards());

  // the currently selected dashboard
  let curDash: Dashboard | null = $state(null as unknown as Dashboard);

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
        asyncDashboards = getDashboards();
        addDashName = "";
        errors.dashboard.add = "";
      })
      .catch((e: APIError) => {
        errors.dashboard.add =
          "Error: " + (e.body as { [key: string]: string }).name;
      });

  const removeDashboard = () => {
    if (curDash) {
      deleteDashboard(curDash)
        .then(() => {
          asyncDashboards = getDashboards();
          curDash = null;
          errors.dashboard.remove = "";
        })
        .catch((e) => {
          errors.dashboard.remove = e.body.toString();
        });
    }
  };
</script>

<div class="flex flex-wrap gap-4 size-full items-stretch">
  <!-- Sidebar -->
  <div class="basis-48 grow bg-base-300 rounded-lg shadow-xs">
    <!-- Dashboard list -->
    <ul class="menu w-full">
      <li class="menu-title">Dashboards</li>
      {#await asyncDashboards}
        Loading...
      {:then dashboards}
        {#each dashboards as d}
          <li>
            <button
              onclick={() => {
                curDash = d;
              }}
              class={{
                "menu-active": curDash?.dashboard_id === d.dashboard_id,
              }}>{d.name}</button
            >
          </li>
        {/each}
      {:catch error}
        <li class="text-error">
          Error{#if error.status}
            ({error.status}){/if}: Could not load dashboards
        </li>
      {/await}
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
    {#if curDash === null}
      <div class="flex flex-col items-center justify-center">
        <h2 class="text-lg font-bold">Select a Dashboard</h2>
      </div>
    {:else}
      {#key curDash}
        <DashboardEditor dashboard={curDash} {removeDashboard} />
        {#if errors.dashboard.remove}
          <p class=" text-error">{errors.dashboard.remove}</p>
        {/if}
      {/key}
    {/if}
  </div>
</div>

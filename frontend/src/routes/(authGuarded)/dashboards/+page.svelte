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
  let curDash: Dashboard | null = $state(null);

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
  <div class="basis-[12rem] grow bg-gray-200 rounded-lg p-3">
    <!-- Dashboard list -->
    <h2>Dashboards</h2>
    <div class="flex flex-col">
      {#await asyncDashboards}
        Loading...
      {:then dashboards}
        {@debug dashboards}
        {#each dashboards as d}
          <button
            onclick={() => {
              curDash = d;
            }}
            class="text-left bg-gray-200 hover:bg-gray-400 transition rounded-xl p-2 mb-2"
            >{d.name}</button
          >
        {/each}
      {:catch error}
        <p class="text-red-500">
          Error{#if error.status}
            ({error.status}){/if}: Could not load dashboards
        </p>
      {/await}
    </div>
    <!-- Dashboard creation input -->
    <div
      class={[
        "rounded-xl py-2 border-2 border-dashed border-gray-400 flex flex-col items-center transition gap-3",
        !addDashMode && "hover:bg-gray-400",
      ]}
    >
      {#if addDashMode}
        <p class="text-center">New Dashboard</p>
        <input bind:value={addDashName} id="table-name-input" />

        <div class="flex gap-3">
          <button
            onclick={() => addDashboard(addDashName)}
            class="px-2 py-1 rounded-lg border-2 border-gray-400 hover:bg-gray-400 transition"
            >Create</button
          >

          <button
            onclick={() => {
              errors.dashboard.add = "";
              addDashName = "";
              addDashMode = false;
            }}
            class="px-2 py-1 rounded-lg border-2 border-red-400 hover:bg-red-400 transition"
            >Cancel</button
          >
        </div>
      {:else}
        <button
          onclick={() => {
            addDashMode = true;
          }}
          class="text-center w-full">Add Dashboard</button
        >
      {/if}
      {#if errors.dashboard.add !== ""}
        <p class="text-red-500">{errors.dashboard.add}</p>
      {/if}
    </div>
  </div>
  <!-- Main editor -->
  <div class="bg-gray-200 basis-[36rem] grow-[5] shrink min-w-0 rounded-lg p-3">
    {#if curDash === null}
      <div class="flex flex-col items-center justify-center">
        <h2 class="text-lg font-bold">Select a Dashboard</h2>
      </div>
    {:else}
      {#key curDash}
        <DashboardEditor dashboard={curDash} {removeDashboard} />
        {#if errors.dashboard.remove}
          <p class=" text-red-500">{errors.dashboard.remove}</p>
        {/if}
      {/key}
    {/if}
  </div>
</div>

<script lang="ts">
  import { createAccess, deleteAccess, patchAccess } from "$lib/api";
  import { type AccessRole, type Access } from "$lib/types/access.js";
  import { type User } from "$lib/types";
  import { refreshAll } from "$app/navigation";

  let {
    curUser,
    allUsers,
    usersWithAccess,
    resource,
    resourceId,
    modal = $bindable(),
  }: {
    curUser: User;
    allUsers?: User[];
    usersWithAccess?: Access[];
    resource: "Table" | "Dashboard";
    resourceId: string;
    modal: any;
  } = $props();

  let remainingUsers = $derived(
    allUsers?.filter(
      (user) =>
        !usersWithAccess?.some(
          (existingUser) => existingUser.username === user.username,
        ),
    ) ?? [],
  );

  const addUserAccess = (username: string, access_role: AccessRole) =>
    createAccess(resource, resourceId, { username, access_role })
      .then(() => refreshAll())
      .catch((e) => {
        error = e.body.toString();
      });

  const changeUserAccess = (username: string, role: AccessRole) =>
    patchAccess(resource, resourceId, username, role).then(() => refreshAll());

  const removeUserAccess = (username: string) =>
    deleteAccess(resource, resourceId, username).then(() => refreshAll());

  let addUserField = $state("");
  let addUserRoleSelect = $state("Editor" as AccessRole);
  let error = $state("");
</script>

<dialog bind:this={modal} class="modal" aria-modal="true">
  {#if usersWithAccess && remainingUsers}
    <div class="modal-box max-w-11/12">
      <h3 class="text-lg font-bold">Manage Access</h3>
      <ul class="list">
        <li>Users with access</li>
        {#each usersWithAccess as u}
          <li class="list-row">
            <div class="flex items-center gap-2">
              {#if u.username === curUser.username}
                <div class="badge badge-outline badge-info">You</div>
              {/if}
              {u.username}
            </div>
            {#if u.access_role === "Owner"}
              <div class="badge badge-soft badge-primary text-xs">owner</div>
              {#if u.username !== curUser.username}
                <div class="join">
                  <div class="join-item btn btn-active btn-xs">Change to:</div>
                  <button
                    class="btn btn-xs join-item"
                    onclick={() => changeUserAccess(u.username, "Editor")}
                    >editor</button
                  >
                  <button
                    class="btn btn-xs join-item"
                    onclick={() => changeUserAccess(u.username, "Viewer")}
                    >viewer</button
                  >
                </div>
              {/if}
            {:else if u.access_role === "Editor"}
              <div class="badge badge-soft badge-secondary text-xs">editor</div>
              <div class="join">
                <div class="join-item btn btn-active btn-xs">Change to:</div>
                <button
                  class="btn btn-xs join-item"
                  onclick={() => changeUserAccess(u.username, "Owner")}
                  >owner</button
                >
                <button
                  class="btn btn-xs join-item"
                  onclick={() => changeUserAccess(u.username, "Viewer")}
                  >viewer</button
                >
              </div>
            {:else}
              <div class="badge badge-soft badge-accent text-xs">viewer</div>
              <div class="join">
                <div class="join-item btn btn-active btn-xs">Change to:</div>
                <button
                  class="btn btn-xs join-item"
                  onclick={() => changeUserAccess(u.username, "Owner")}
                  >owner</button
                >
                <button
                  class="btn btn-xs join-item"
                  onclick={() => changeUserAccess(u.username, "Editor")}
                  >editor</button
                >
              </div>
            {/if}
            {#if u.username !== curUser.username}
              <button
                class="btn btn-error btn-xs"
                aria-label="remove"
                onclick={() => removeUserAccess(u.username)}>X</button
              >
            {/if}
          </li>
        {/each}
      </ul>
      <div>
        <p class="font-bold">Add user:</p>
        <div class="w-full flex justify-center">
          <div class="join w-full">
            <input
              class="input join-item"
              name="username"
              title="username"
              placeholder="username"
              bind:value={addUserField}
              list="users-list"
            />
            <datalist id="users-list">
              {#each remainingUsers as u}
                <option value={u.username}></option>
              {/each}
            </datalist>
            <select
              name="role"
              title="role"
              class="select join-item"
              bind:value={addUserRoleSelect}
            >
              <option value="Owner">Owner</option>
              <option selected value="Editor">Editor</option>
              <option value="Viewer">Viewer</option>
            </select>
            <button
              class="btn join-item"
              onclick={() => addUserAccess(addUserField, addUserRoleSelect)}
              >Add</button
            >
          </div>
        </div>
        {#if error}
          <p class="text-error mt-2">{error}</p>
        {/if}
      </div>
    </div>
  {/if}
  <form method="dialog" class="modal-backdrop">
    <button
      onclick={() => {
        error = "";
      }}>close</button
    >
  </form>
</dialog>

<script lang="ts">
  import { onMount } from "svelte";
  import {
    getAllUsers,
    createUser,
    deleteUser,
    updateUser,
    type UserResponse,
    type CreateUser,
    type UpdateUser,
  } from "$lib/api/userManagement";

  let users: UserResponse[] = [];
  let loading = false;
  let error: string | null = null;
  let success: string | null = null;

  // Create form
  let newUsername = "";
  let newPassword = "";

  // Edit form
  let editingId: number | null = null;
  let originalUsername = "";
  let editUsername = "";
  let editPassword = "";

  let q = "";

  async function loadUsers() {
    loading = true;
    try {
      users = await getAllUsers();
    } catch (e: any) {
      error = e?.message || "Failed to load users.";
    } finally {
      loading = false;
    }
  }

  function validateCreate(): string | null {
    if (!newUsername.trim()) return "Username is required.";
    if (!newUsername.includes("@")) return "Username must contain '@'.";
    if (!newPassword.trim()) return "Password is required.";
    if (newPassword.length < 6)
      return "Password must be at least 6 characters.";
    return null;
  }

  function validateEdit(): string | null {
    if (editUsername && !editUsername.includes("@"))
      return "Username must contain '@'.";
    if (editPassword && editPassword.length < 6)
      return "Password must be at least 6 characters.";
    return null;
  }

  async function handleCreateUser() {
    const validation = validateCreate();
    if (validation) {
      error = validation;
      return;
    }

    loading = true;
    error = null;
    success = null;

    try {
      const payload: CreateUser = {
        username: newUsername.trim(),
        password: newPassword,
      };

      await createUser(payload);
      success = `User "${payload.username}" created.`;

      newUsername = "";
      newPassword = "";
      await loadUsers();
    } catch (e: any) {
      error = e?.response?.data?.message || "Failed to create user.";
    } finally {
      loading = false;
      setTimeout(() => (success = null), 2500);
    }
  }

  function startEdit(user: UserResponse) {
    editingId = user.user_id;
    originalUsername = user.username; // Needed to detect if username changed
    editUsername = user.username;
    editPassword = "";
    error = null;
    success = null;
  }

  async function handleUpdateUser() {
    if (!editingId) return;

    const validation = validateEdit();
    if (validation) {
      error = validation;
      return;
    }

    // Only send the fields that the user CHANGED
    const payload: UpdateUser = {
      username:
        editUsername.trim() !== originalUsername ? editUsername.trim() : null,
      password: editPassword ? editPassword : null,
    };

    if (!payload.username && !payload.password) {
      error = "You did not modify anything.";
      return;
    }

    loading = true;
    error = null;
    success = null;

    try {
      await updateUser(editingId, payload);
      success = "User updated.";

      editingId = null;
      editUsername = "";
      editPassword = "";
      await loadUsers();
    } catch (e: any) {
      error = e?.response?.data?.message || "Failed to update user.";
    } finally {
      loading = false;
      setTimeout(() => (success = null), 2500);
    }
  }

  async function handleDeleteUser(id: number, username: string) {
    const confirmation = confirm(`Delete user "${username}"?`);
    if (!confirmation) return;

    loading = true;
    error = null;
    success = null;

    try {
      await deleteUser(id);
      success = `User "${username}" deleted.`;
      await loadUsers();
    } catch (e: any) {
      error = e?.response?.data?.message || "Failed to delete user.";
    } finally {
      loading = false;
      setTimeout(() => (success = null), 2500);
    }
  }

  $: filtered = q
    ? users.filter(
        (u) =>
          u.username.toLowerCase().includes(q.toLowerCase()) ||
          String(u.user_id) === q,
      )
    : users;

  onMount(loadUsers);
</script>

<!-- No custom CSS needed – DaisyUI handles everything -->

<div class="max-w-5xl mx-auto p-6">
  <h1 class="text-3xl font-bold mb-6">User Management</h1>

  {#if loading}
    <progress class="progress w-full"></progress>
  {/if}

  {#if error}
    <div class="alert alert-error mb-4">{error}</div>
  {/if}

  {#if success}
    <div class="alert alert-success mb-4">{success}</div>
  {/if}

  <!-- CREATE USER -->
  <div class="card bg-base-200 shadow-md p-5 mb-8">
    <h2 class="text-xl font-bold mb-3">Create User</h2>

    <div class="flex flex-col md:flex-row gap-3">
      <input
        class="input input-bordered w-full"
        placeholder="Username"
        bind:value={newUsername}
      />

      <input
        type="password"
        class="input input-bordered w-full"
        placeholder="Password"
        bind:value={newPassword}
      />

      <button
        class="btn btn-primary"
        on:click={handleCreateUser}
        disabled={loading}
      >
        Create
      </button>
    </div>
  </div>

  <!-- SEARCH -->
  <input
    class="input input-bordered w-full mb-4"
    placeholder="Search by username or ID"
    bind:value={q}
  />

  <!-- USER TABLE -->
  <div class="overflow-x-auto">
    <table class="table table-zebra w-full">
      <thead>
        <tr>
          <th></th>
          <th>Username</th>
          <th>Admin</th>
          <th>Actions</th>
        </tr>
      </thead>

      <tbody>
        {#each filtered as user}
          <tr>
            <td>{user.user_id}</td>

            <td>
              {#if editingId === user.user_id}
                <input
                  class="input input-bordered w-full mb-2"
                  bind:value={editUsername}
                />
                <input
                  type="password"
                  class="input input-bordered w-full"
                  placeholder="New password (optional)"
                  bind:value={editPassword}
                />
              {:else}
                {user.username}
              {/if}
            </td>

            <td>
              {#if user.is_admin}
                <div class="badge badge-primary">Admin</div>
              {:else}
                <div class="badge badge-secondary">User</div>
              {/if}
            </td>

            <td class="flex gap-2">
              {#if editingId === user.user_id}
                <button
                  class="btn btn-success btn-sm"
                  on:click={handleUpdateUser}
                >
                  Save
                </button>
                <button
                  class="btn btn-neutral btn-sm"
                  on:click={() => (editingId = null)}
                >
                  Cancel
                </button>
              {:else}
                <button
                  class="btn btn-info btn-sm"
                  on:click={() => startEdit(user)}
                >
                  Edit
                </button>

                <button
                  class="btn btn-error btn-sm"
                  on:click={() => handleDeleteUser(user.user_id, user.username)}
                >
                  Delete
                </button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

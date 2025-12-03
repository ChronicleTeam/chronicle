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
  
    // Users list
    let users: UserResponse[] = [];
    let loading = false;
    let error: string | null = null;
    let success: string | null = null;
  
    // Create form
    let newUsername = "";
    let newPassword = "";
  
    // Edit form
    let editingId: number | null = null;
    let editUsername = "";
    let editPassword = "";
  
    // Search query
    let q = "";
  
    /** Load all users */
    async function loadUsers() {
      loading = true;
      error = null;
      try {
        users = await getAllUsers();
        console.log("Loaded users:", users);
      } catch (e: any) {
        console.error("Failed to load users:", e);
        error = e?.message || "Failed to load users.";
      } finally {
        loading = false;
      }
    }
  
    /** Validate create form */
    function validateCreate(): string | null {
      if (!newUsername.trim()) return "Username is required.";
      if (!newUsername.includes("@")) return "Username must contain '@'.";
      if (!newPassword.trim()) return "Password is required.";
      if (newPassword.length < 6) return "Password must be at least 6 characters.";
      return null;
    }
  
    /** Validate edit form */
    function validateEdit(): string | null {
      if (editUsername && !editUsername.includes("@")) return "Username must be of the form 'test@example.com'.";
      if (editPassword && editPassword.length < 6) return "Password must be at least 6 characters.";
      return null;
    }
  
    /** Create user */
    async function handleCreateUser() {
      const validation = validateCreate();
      if (validation) {
        error = validation;
        return;
      }
  
      loading = true;
      error = null;
      success = null;
  
      const payload: CreateUser = {
        username: newUsername.trim(),
        password: newPassword,
      };
  
      try {
        console.log("Creating user:", payload);
        await createUser(payload);
        success = `User "${payload.username}" created successfully.`;
        newUsername = "";
        newPassword = "";
        await loadUsers();
      } catch (e: any) {
        console.error("Create user error:", e);
        error = e?.response?.data?.message || "Failed to create user.";
      } finally {
        loading = false;
        setTimeout(() => (success = null), 2500);
      }
    }
  
    /**  editing a user */
    function startEdit(user: UserResponse) {
      editingId = user.user_id;
      editUsername = user.username;
      editPassword = "";
      error = null;
      success = null;
    }
  
    /** Update user */
    async function handleUpdateUser() {
      if (!editingId) return;
  
      const validation = validateEdit();
      if (validation) {
        error = validation;
        return;
      }
  
      const payload: UpdateUser = {
        username: editUsername?.trim() || null,
        password: editPassword || null,
      };
  
      if (!payload.username && !payload.password) {
        error = "No changes to apply.";
        return;
      }
  
      loading = true;
      error = null;
      success = null;
  
      try {
        console.log("Updating user:", editingId, payload);
        await updateUser(editingId, payload);
        success = "User updated successfully.";
        editingId = null;
        editUsername = "";
        editPassword = "";
        await loadUsers();
      } catch (e: any) {
        console.error("Update user error:", e);
        error = e?.response?.data?.message || "Failed to update user.";
      } finally {
        loading = false;
        setTimeout(() => (success = null), 2500);
      }
    }
  
    /** Delete user */
    async function handleDeleteUser(id: number, username: string) {
      const confirmation = confirm(`Are you sure you want to delete user "${username}"?`);
      if (!confirmation) return;
  
      loading = true;
      error = null;
      success = null;
  
      try {
        console.log("Deleting user:", id);
        await deleteUser(id);
        success = `User "${username}" deleted successfully.`;
        await loadUsers();
      } catch (e: any) {
        console.error("Delete user error:", e);
        error = e?.response?.data?.message || "Failed to delete user.";
      } finally {
        loading = false;
        setTimeout(() => (success = null), 2500);
      }
    }
  
    /** Filter users based on search */
    $: filtered = q
      ? users.filter(
          (u) =>
            u.username.toLowerCase().includes(q.toLowerCase()) ||
            String(u.user_id) === q
        )
      : users;
  
    onMount(loadUsers);
  </script>
  
  <style>
    .container {
      max-width: 900px;
      margin: 20px auto;
      font-family: sans-serif;
      color: white;
    }
  
    table {
      width: 100%;
      border-collapse: collapse;
      margin-top: 10px;
      color: white;
    }
  
    th, td {
      padding: 8px;
      border-bottom: 1px solid #ddd;
      text-align: left;
    }
  
    input, button {
      padding: 6px 8px;
      margin-right: 8px;
      background: #222;
      border: 1px solid #555;
      color: white;
    }
  
    input::placeholder {
      color: #888;
    }
  
    .card {
      padding: 10px;
      border-radius: 6px;
      margin-top: 10px;
    }
  
    .success {
      background: #0a6a0a;
      color: #e7ffe7;
    }
  
    .error {
      background: #a30000;
      color: #ffe5e5;
    }
  
    .badge {
      padding: 2px 6px;
      border-radius: 4px;
      font-size: 0.85em;
    }
  
    .admin {
      background: #007bff;
      color: white;
    }
  
    .user {
      background: #aaa;
      color: white;
    }
  
    .actions button {
      margin-right: 6px;
    }
  
    button:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
  </style>
  
  <div class="container">
    <h1>User Management</h1>
  
    {#if loading}
      <p>Loading...</p>
    {/if}
  
    {#if error}
      <div class="card error">{error}</div>
    {/if}
  
    {#if success}
      <div class="card success">{success}</div>
    {/if}
  
    <!-- CREATE FORM -->
    <h2>Create User</h2>
    <div>
      <input placeholder="Username" bind:value={newUsername} />
      <input type="password" placeholder="Password" bind:value={newPassword} />
      <button on:click={handleCreateUser} disabled={loading}>Create</button>
    </div>
  
    <hr />
  
    <!-- SEARCH -->
    <h2>User List</h2>
    <input placeholder="Search by username or id" bind:value={q} style="margin-bottom:10px;" />
  
    <!-- USER TABLE -->
    <table>
      <thead>
        <tr>
          <th>ID</th>
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
                <input bind:value={editUsername} />
                <br />
                <input type="password" placeholder="New password (optional)" bind:value={editPassword} />
              {:else}
                {user.username}
              {/if}
            </td>
            <td>
              {#if user.is_admin}
                <span class="badge admin">Admin</span>
              {:else}
                <span class="badge user">User</span>
              {/if}
            </td>
            <td class="actions">
              {#if editingId === user.user_id}
                <button on:click={handleUpdateUser}>Save</button>
                <button on:click={() => (editingId = null)}>Cancel</button>
              {:else}
                <button on:click={() => startEdit(user)}>Edit</button>
                <button on:click={() => handleDeleteUser(user.user_id, user.username)}>Delete</button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  
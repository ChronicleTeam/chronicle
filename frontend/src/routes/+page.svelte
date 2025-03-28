<script lang="ts">
  import { goto } from "$app/navigation";
  import type { APIError } from "$lib/api";
  import type { Credentials } from "$lib/types";
  import { login } from "$lib/user.svelte";

  let credentials: Credentials = $state({
    username: "",
    password: "",
  });

  let error = $state("");

  async function handleLogin() {
    try {
      // Send login request
      await login(credentials);
      error = "";
      goto("/tables"); // Redirect to tables page on success
    } catch (e) {
      console.error("Login error:", e);
      error = (e as unknown as APIError).body.toString();
    }
  }

  function goToRegister() {
    goto("/register");
  }

  function goToTables() {
    goto("/tables");
  }
</script>

<div
  class="flex flex-col justify-center items-center h-screen gap-6 bg-gray-100"
>
  <img src="/logo.png" alt="Logo" class="h-20 mb-4" />
  <h1 class="text-5xl font-bold text-center">Chronicle</h1>
  <p class="text-lg text-center text-gray-600">Data analysis made simple.</p>

  <div
    class="flex flex-col items-center bg-white p-6 rounded-lg shadow-lg w-80"
  >
    <form onsubmit={handleLogin} class="w-full">
      <div class="mb-4">
        <label for="email" class="block text-sm font-semibold">Email</label>
        <input
          type="email"
          id="email"
          bind:value={credentials.username}
          placeholder="Enter your email"
          class="w-full p-2 mt-1 border rounded-md"
          required
        />
      </div>

      <div class="mb-6">
        <label for="password" class="block text-sm font-semibold"
          >Password</label
        >
        <input
          type="password"
          id="password"
          bind:value={credentials.password}
          placeholder="Enter your password"
          class="w-full p-2 mt-1 border rounded-md"
          required
        />
      </div>
      {#if error}
        <p class="text-red-500">{error}</p>
      {/if}

      <button
        type="submit"
        class="w-full py-2 bg-sky-700 text-white hover:bg-sky-900 rounded-md transition-all duration-300"
      >
        Login
      </button>

      <button
        type="button"
        onclick={goToRegister}
        class="w-full mt-3 py-2 bg-gray-300 text-black hover:bg-gray-400 rounded-md transition-all duration-300"
      >
        Register
      </button>

      <button
        type="button"
        onclick={goToTables}
        class="w-full mt-3 py-2 bg-gray-300 text-black hover:bg-gray-400 rounded-md transition-all duration-300"
      >
        Go to tables
      </button>
    </form>
  </div>
</div>

<script lang="ts">
  import { goto } from "$app/navigation";
  import type { APIError } from "$lib/api";
  import type { Credentials } from "$lib/types";
  import { login, user } from "$lib/user.svelte";
  import { onMount } from "svelte";

  const SUCCESS_REDIRECT = "/tables";

  let credentials: Credentials = $state({
    username: "",
    password: "",
  });

  let error = $state("");

  async function handleLogin(e: SubmitEvent) {
    e.preventDefault();
    try {
      // Send login request
      await login(credentials);
      error = "";
      await goto(SUCCESS_REDIRECT); // Redirect to tables page on success
    } catch (e) {
      error = (e as unknown as APIError).body.toString();
    }
  }

  onMount(() => {
    user().then((u) => {
      if (u) goto(SUCCESS_REDIRECT);
    });
  });
</script>

<div
  class="flex flex-col justify-center items-center h-screen gap-6 bg-base-200"
>
  <!-- Header -->
  <img src="/logo.png" alt="Logo" class="h-20 mb-4" />
  <h1 class="text-5xl font-bold text-center">Chronicle</h1>
  <p class="text-lg text-center text-base-content opacity-70">
    Data analysis made simple.
  </p>

  <!-- Login form -->
  <div class="card bg-base-100 shadow-lg w-80">
    <form onsubmit={handleLogin} class="card-body flex flex-col gap-4">
      <div>
        <label for="email" class="block text-sm font-semibold">Email</label>
        <input
          type="email"
          id="email"
          bind:value={credentials.username}
          placeholder="Enter your email"
          class="input"
          required
        />
      </div>

      <div>
        <label for="password" class="block text-sm font-semibold"
          >Password</label
        >
        <input
          type="password"
          id="password"
          bind:value={credentials.password}
          placeholder="Enter your password"
          class="input"
          required
        />
      </div>
      {#if error}
        <p class="text-error">{error}</p>
      {/if}

      <button type="submit" class="btn btn-primary btn-block"> Login </button>
    </form>
  </div>
</div>

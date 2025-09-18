<script lang="ts">
  import { goto } from "$app/navigation";

  let email = "";
  let password = "";
  let confirmPassword = "";

  async function handleRegister() {
    if (password !== confirmPassword) {
      alert("Passwords do not match!");
      return;
    }

    try {
      // Send registration request
      const response = await fetch(`/api/users/${encodeURIComponent(email)}`, {
        method: "PUT", // Assume user is created with PUT (or use POST if required)
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ password }),
      });

      const data = await response.json();
      if (response.ok) {
        alert("Registration successful! You can now log in.");
        goto("/"); // Redirect to login page
      } else {
        alert(data.message || "Registration failed!");
      }
    } catch (error) {
      console.error("Registration error:", error);
      alert("An error occurred. Please try again.");
    }
  }
</script>

<div
  class="flex flex-col justify-center items-center h-screen gap-6 bg-base-200"
>
  <img src="/logo.png" alt="Logo" class="h-20 mb-4" />
  <div class="card bg-base-100 shadow-lg w-80">
    <form
      on:submit|preventDefault={handleRegister}
      class="card-body flex flex-col gap-4"
    >
      <h2 class="card-title text-2xl font-semibold mb-4">Register</h2>
      <div>
        <label for="email" class="block text-sm font-semibold">Email</label>
        <input
          type="email"
          id="email"
          bind:value={email}
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
          bind:value={password}
          placeholder="Enter a password"
          class="input"
          required
        />
      </div>

      <div>
        <label for="confirmPassword" class="block text-sm font-semibold"
          >Confirm Password</label
        >
        <input
          type="password"
          id="confirmPassword"
          bind:value={confirmPassword}
          placeholder="Confirm your password"
          class="input"
          required
        />
      </div>

      <button type="submit" class="btn btn-primary btn-block">
        Register
      </button>

      <button type="button" on:click={() => goto("/")} class="btn btn-block">
        Back to Login
      </button>
    </form>
  </div>
</div>

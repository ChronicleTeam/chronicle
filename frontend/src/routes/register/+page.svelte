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
  class="flex flex-col justify-center items-center h-screen gap-6 bg-gray-100"
>
  <img src="/logo.png" alt="Logo" class="h-20 mb-4" />
  <div
    class="flex flex-col items-center bg-white p-6 rounded-lg shadow-lg w-80"
  >
    <h2 class="text-2xl font-semibold mb-4">Register</h2>

    <form on:submit|preventDefault={handleRegister} class="w-full">
      <div class="mb-4">
        <label for="email" class="block text-sm font-semibold">Email</label>
        <input
          type="email"
          id="email"
          bind:value={email}
          placeholder="Enter your email"
          class="w-full p-2 mt-1 border rounded-md"
          required
        />
      </div>

      <div class="mb-4">
        <label for="password" class="block text-sm font-semibold"
          >Password</label
        >
        <input
          type="password"
          id="password"
          bind:value={password}
          placeholder="Enter a password"
          class="w-full p-2 mt-1 border rounded-md"
          required
        />
      </div>

      <div class="mb-6">
        <label for="confirmPassword" class="block text-sm font-semibold"
          >Confirm Password</label
        >
        <input
          type="password"
          id="confirmPassword"
          bind:value={confirmPassword}
          placeholder="Confirm your password"
          class="w-full p-2 mt-1 border rounded-md"
          required
        />
      </div>

      <button
        type="submit"
        class="w-full py-2 bg-sky-700 text-white hover:bg-sky-900 rounded-md transition-all duration-300"
      >
        Register
      </button>

      <button
        type="button"
        on:click={() => goto("/")}
        class="w-full mt-3 py-2 bg-gray-300 text-black hover:bg-gray-400 rounded-md transition-all duration-300"
      >
        Back to Login
      </button>
    </form>
  </div>
</div>


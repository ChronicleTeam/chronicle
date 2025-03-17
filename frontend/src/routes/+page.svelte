<script lang="ts">
  import { goto } from "$app/navigation";

  let email = "";
  let password = "";

  async function handleLogin() {
    try {
      // Send login request 
      const response = await fetch(`/api/users/${encodeURIComponent(email)}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ password }),
      });

      const data = await response.json();
      if (response.ok) {
        alert("Login successful!");
        goto("/tables"); // Redirect to tables page on success
      } else {
        alert(data.message || "Login failed!");
      }
    } catch (error) {
      console.error("Login error:", error);
      alert("An error occurred. Please try again.");
    }
  }

  function goToRegister() {
    goto("/register");
  }
  
  function goToTables() {
    goto("/tables");
  }
</script>

<div class="flex flex-col justify-center items-center h-screen gap-6 bg-gray-100">
  <img src="/logo.png" alt="Logo" class="h-20 mb-4" />
  <h1 class="text-5xl font-bold text-center">Chronicle</h1>
  <p class="text-lg text-center text-gray-600">Data analysis made simple.</p>

  <div class="flex flex-col items-center bg-white p-6 rounded-lg shadow-lg w-80">
    <form on:submit|preventDefault={handleLogin} class="w-full">
      <div class="mb-4">
        <label for="email" class="block text-sm font-semibold">Email</label>
        <input type="email" id="email" bind:value={email} placeholder="Enter your email" class="w-full p-2 mt-1 border rounded-md" required />
      </div>

      <div class="mb-6">
        <label for="password" class="block text-sm font-semibold">Password</label>
        <input type="password" id="password" bind:value={password} placeholder="Enter your password" class="w-full p-2 mt-1 border rounded-md" required />
      </div>

      <button type="submit" class="w-full py-2 bg-sky-700 text-white hover:bg-sky-900 rounded-md transition-all duration-300">
        Login
      </button>

      <button type="button" on:click={goToRegister} class="w-full mt-3 py-2 bg-gray-300 text-black hover:bg-gray-400 rounded-md transition-all duration-300">
        Register
      </button>
      
      <button type="button"on:click={goToTables} class="w-full mt-3 py-2 bg-gray-300 text-black hover:bg-gray-400 rounded-md transition-all duration-300">
        Go to tables
      </button>

    </form>
  </div>
</div>

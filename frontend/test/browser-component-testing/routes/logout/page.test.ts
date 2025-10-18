// @ts-ignore
import { goto } from "$app/navigation";
import { expect, vi, afterEach, describe } from 'vitest'
import { page } from '@vitest/browser/context'

import Logout from '../../../../src/routes/logout/+page.svelte'
import { it } from '../../test-extensions';
import { clearUser, login, user } from '../../../../src/lib/user.svelte';

describe("logout page", () => {
  it.scoped({ authenticated: false });
  afterEach(async () => {
    vi.clearAllMocks();
  })

  it('logs out and redirects', async () => {
    await clearUser();
    await login({ username: "test@example.com", password: "test123" });
    const screen = page.render(Logout);

    await expect.element(screen.getByText("Logged out.")).toBeVisible();
    await expect.poll(() => goto).toHaveBeenCalledExactlyOnceWith("/");
    await expect(user()).resolves.toBeNull()
  })
})


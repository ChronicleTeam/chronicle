// @ts-ignore
import { goto } from "$app/navigation";
import { expect, vi, afterEach, describe } from 'vitest'
import { http, HttpResponse } from 'msw'
import { page } from '@vitest/browser/context'

import Login from '../../../src/routes/+page.svelte'
import { it } from '../test-extensions';
import { clearUser } from '../../../src/lib/user.svelte'

describe("login page", () => {
  it.scoped({ authenticated: false });

  afterEach(async () => {
    vi.clearAllMocks();
    await clearUser();
  })

  it('renders name', async () => {
    const screen = page.render(Login)
    await expect.element(screen.getByText('Chronicle', { exact: true })).toBeVisible()
  })

  it('logs in properly', async ({ authenticated, worker }) => {
    const screen = page.render(Login)

    const emailInput = screen.getByRole('textbox', { name: 'email' })
    const password = screen.getByRole('textbox', { name: 'password' })
    const loginButton = screen.getByRole('button', { name: 'login' })

    await emailInput.fill('test@example.com')
    await password.fill('test123')

    await expect.element(loginButton).toBeVisible()
    await loginButton.click()

    await expect.poll(() => goto).toHaveBeenCalledExactlyOnceWith('/tables');
  })

  it('handles log in error gracefully', async () => {
    const screen = page.render(Login)

    const emailInput = screen.getByRole('textbox', { name: 'email' })
    const password = screen.getByRole('textbox', { name: 'password' })
    const loginButton = screen.getByRole('button', { name: 'login' })

    await emailInput.fill('test@example.com')
    await password.fill('test124')

    await expect.element(loginButton).toBeVisible()

    await loginButton.click()

    await expect.element(screen.getByText("Invalid credentials")).toBeVisible()

    await expect.poll(() => goto).not.toHaveBeenCalled();
  })

  it('redirects if user already logged in', async ({ worker }) => {
    worker.use(
      http.get('https://www.example.com/api/user', () => {
        return HttpResponse.json(
          {
            user_id: 1,
            username: 'test',
          }
        )
      })
    );

    page.render(Login);

    await expect.poll(() => goto).toHaveBeenCalledExactlyOnceWith('/tables');
  })
});

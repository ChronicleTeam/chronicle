// @ts-ignore
import { goto } from "$app/navigation";
import { expect, vi, afterEach, describe } from 'vitest'
import { page } from '@vitest/browser/context'

import Layout from '../../../../src/routes/(authGuarded)/+layout.svelte'
import { it } from '../../test-extensions';
import { createRawSnippet } from 'svelte';

const mockChild = createRawSnippet(() => ({
  render: () => `<h1>This is a child component</h1>`,
}))
describe("main authenticated layout", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  })

  describe("when not authenticated", () => {
    it.scoped({ authenticated: false })
    it('blocks user', async () => {
      const screen = page.render(Layout, { props: { children: mockChild } });

      await expect.element(screen.getByRole("heading", { name: "not authorized" })).toBeVisible();
      await expect.element(screen.getByRole("link", { name: "go home" })).toBeVisible();
    })
  })

  it("renders children", async () => {
    const screen = page.render(Layout, { props: { children: mockChild } });
    await expect.element(screen.getByRole("heading", { name: "This is a child component", exact: true })).toBeVisible();
  })

  it("renders navigation bar", async () => {
    const screen = page.render(Layout, { props: { children: mockChild } });

    await expect.element(screen.getByRole("heading", { name: "chronicle" })).toBeVisible();

    const menuButton = screen.getByRole("button", { name: "navigation menu" })
    await expect.element(menuButton).toBeVisible();
    await menuButton.click();

    await expect.element(screen.getByRole("link", { name: "data management" })).toBeVisible();
    await expect.element(screen.getByRole("link", { name: "dashboards" })).toBeVisible();
    await expect.element(screen.getByRole("link", { name: "logout" })).toBeVisible();
  })
})



// @ts-ignore
import { expect, vi, afterEach, describe } from 'vitest'
import { page } from '@vitest/browser/context'

import Layout from '../../../src/routes/+layout.svelte'
import { it } from '../test-extensions';
import { createRawSnippet } from 'svelte';


const mockChild = createRawSnippet(() => ({
  render: () => `<h1>This is a child component</h1>`,
}))

describe("root layout", () => {
  it.scoped({ authenticated: false });

  afterEach(async () => {
    vi.clearAllMocks();
  });

  it('renders children', async () => {
    const screen = page.render(Layout, { props: { children: mockChild } })
    await expect.element(screen.getByRole("heading", { name: "This is a child component", exact: true })).toBeVisible();
  });
})

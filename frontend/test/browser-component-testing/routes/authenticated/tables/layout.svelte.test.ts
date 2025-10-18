// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { goto } from "$app/navigation";
import { expect, vi, afterEach, describe } from 'vitest'
import { page } from '@vitest/browser/context'

import Layout from '../../../../../src/routes/(authGuarded)/tables/+layout.svelte'
import { it } from '../../../test-extensions';
import { createRawSnippet } from 'svelte';

const mockChild = createRawSnippet(() => ({
  render: () => `<h1>This is a child component</h1>`,
}))


describe("main table layout", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  })

  it("renders children", async () => {
    let testData = $state({
      tables: [
        {
          table_id: 123,
          user_id: 456,
          name: "Test Table",
          description: "Description"
        }
      ]
    })
    const screen = page.render(Layout, { props: { children: mockChild, data: testData } });
    await expect.element(screen.getByRole("heading", { name: "This is a child component", exact: true })).toBeVisible();
  })

  it("renders table list", async () => {
    let testData = $state({
      tables: [
        {
          table_id: 123,
          user_id: 456,
          name: "Test Table",
          description: "Description"
        },
        {
          table_id: 124,
          user_id: 456,
          name: "Test Table 2",
          description: "Description"
        }
      ]
    })
    const screen = page.render(Layout, { props: { children: mockChild, data: testData } });
    await expect.element(screen.getByRole("menuitem", { name: "Test Table" }).first()).toBeVisible();
    await expect.element(screen.getByRole("menuitem", { name: "Test Table 2" })).toBeVisible();
  })

})




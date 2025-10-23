// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { goto } from "$app/navigation";
import { expect, vi, afterEach, describe } from 'vitest'
import { page } from '@vitest/browser/context'

import Layout from '../../../../../src/routes/(authGuarded)/tables/+layout.svelte'
import { it } from '../../../test-extensions';
import { createRawSnippet } from 'svelte';
import { postCreateTable } from '../../../../../src/lib/api/dataManagement.ts';

vi.mock('../../../../../src/lib/api/dataManagement.ts', { spy: true })

const mockChild = createRawSnippet(() => ({
  render: () => `<h1>This is a child component</h1>`,
}))


describe("main table layout", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  })

  it("renders children", async () => {
    let testData = {
      tables: [
        {
          table_id: 123,
          user_id: 456,
          name: "Test Table",
          description: "Description"
        }
      ]
    }
    const screen = page.render(Layout, { props: { children: mockChild, data: testData } });
    await expect.element(screen.getByRole("heading", { name: "This is a child component", exact: true })).toBeVisible();
  })

  it("renders table list", async () => {
    let testData = $state({
      tables: [
        {
          table_id: 123,
          user_id: 456,
          name: "Test Table 1",
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
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 1" })).toBeVisible();
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 2" })).toBeVisible();

    testData.tables.push({
      table_id: 125,
      user_id: 456,
      name: "Test Table 3",
      description: "Description"
    })
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 1" })).toBeVisible();
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 2" })).toBeVisible();
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 3" })).toBeVisible();
  });

  it("creates a table", async () => {
    let testData = $state({
      tables: [
        {
          table_id: 123,
          user_id: 456,
          name: "Test Table 1",
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
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 1" })).toBeVisible();
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 2" })).toBeVisible();

    await screen.getByRole("checkbox", { name: "add table" }).click();

    const createInput = screen.getByRole("textbox", { name: "table name" });
    await expect.element(createInput).toBeVisible();
    await createInput.fill("My New Table");

    await screen.getByRole("button", { name: "create" }).click()

    await expect.poll(() => postCreateTable).toHaveBeenCalledOnce()
    await expect.poll(() => postCreateTable).toHaveResolvedWith({
      table_id: 1,
      user_id: 456,
      name: "A New Table",
      description: "a description"
    })
  });

  it("imports a table", async () => {
    let testData = $state({
      tables: [
        {
          table_id: 123,
          user_id: 456,
          name: "Test Table 1",
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
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 1" })).toBeVisible();
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 2" })).toBeVisible();

    await screen.getByRole("checkbox", { name: "add table" }).click();

    // TODO: Look for file input, then use userEvent.upload() to upload a file
    // Then check as usual
  })
})




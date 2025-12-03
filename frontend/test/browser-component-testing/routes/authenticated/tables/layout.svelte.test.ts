// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { expect, vi, afterEach, describe } from 'vitest'
import { page, userEvent } from '@vitest/browser/context'

import Layout from '../../../../../src/routes/(authGuarded)/tables/+layout.svelte'
import { it } from '../../../test-extensions';
import { createRawSnippet } from 'svelte';
import { postCreateTable, postImportTable } from '../../../../../src/lib/api/dataManagement.ts';
import { http, HttpResponse } from "msw";
import { load } from '../../../../../src/routes/(authGuarded)/tables/+layout.ts'


vi.mock('../../../../../src/lib/api/dataManagement.ts', { spy: true })
vi.mock("$app/state", () => ({
  page: {
    params: {
      table_id: "124"
    }
  }
}));



const mockChild = createRawSnippet(() => ({
  render: () => `<h1>This is a child component</h1>`,
}));


describe("main table layout", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  })

  it("renders children", async () => {
    let testData = {
      tables: [
        {
          access_role: "Owner",
          table: {
            table_id: 123,
            user_id: 456,
            name: "Test Table",
            description: "Description"
          }
        }
      ]
    };
    const screen = page.render(Layout, { props: { children: mockChild, data: testData } });
    await expect.element(screen.getByRole("heading", { name: "This is a child component", exact: true })).toBeVisible();
  })

  it("renders table list", async () => {
    const loadResult = await load({ params: {} })
    const data = $state(loadResult)
    const screen = page.render(Layout, { props: { children: mockChild, data } });

    const table1ListItem = screen.getByRole("link", { name: "Test Table 1" });
    const table2ListItem = screen.getByRole("link", { name: "Test Table 2" });

    await expect.element(table1ListItem).toBeVisible();
    await expect.element(table1ListItem).not.toHaveClass("menu-active");
    await expect.element(table2ListItem).toBeVisible();
    await expect.element(table2ListItem).toHaveClass("menu-active");

    data.tables.push(
      {
        access_role: "Owner",
        table: {
          table_id: 125,
          user_id: 456,
          name: "Test Table 3",
          description: "Description"
        }
      }
    );

    const table3ListItem = screen.getByRole("link", { name: "Test Table 3" });

    await expect.element(table1ListItem).toBeVisible();
    await expect.element(table1ListItem).not.toHaveClass("menu-active");
    await expect.element(table2ListItem).toBeVisible();
    await expect.element(table2ListItem).toHaveClass("menu-active");
    await expect.element(table3ListItem).toBeVisible();
    await expect.element(table3ListItem).not.toHaveClass("menu-active");
  });

  it("renders table list with subtable", async () => {
    const loadResult = await load({ params: { subtable_id: 999 } })
    const data = $state(loadResult)
    const screen = page.render(Layout, { props: { children: mockChild, data } });

    const table1ListItem = screen.getByRole("link", { name: "Test Table 1" });
    const table2ListItem = screen.getByRole("link", { name: "Test Table 2" });

    await expect.element(table1ListItem).toBeVisible();
    await expect.element(table1ListItem).not.toHaveClass("menu-active");
    await expect.element(table2ListItem).toBeVisible();
    await expect.element(table2ListItem).not.toHaveClass("menu-active");

    const subtableListItem = screen.getByText("Test Subtable");
    await expect.element(subtableListItem).toBeVisible();
    await expect.element(subtableListItem).toHaveClass("menu-active");
  });


  it("creates a table", async () => {
    const loadResult = await load({ params: {} })
    const data = $state(loadResult)
    const screen = page.render(Layout, { props: { children: mockChild, data } });
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
    const loadResult = await load({ params: {} })
    const data = $state(loadResult)
    const screen = page.render(Layout, { props: { children: mockChild, data } });
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 1" })).toBeVisible();
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 2" })).toBeVisible();

    await screen.getByRole("checkbox", { name: "add table" }).click();

    const fileInput = screen.getByLabelText("file input");
    const fileImportButton = screen.getByRole("button", { name: "import" });

    await expect.element(fileInput).toBeVisible();
    await expect.element(fileImportButton).toBeVisible();

    const testFile = new File(["This, is, a, test, file"], "test.csv");

    await userEvent.upload(fileInput, testFile);
    await fileImportButton.click();

    await expect.poll(() => postImportTable).toHaveBeenCalledExactlyOnceWith(testFile);
    await expect.poll(() => postImportTable).toHaveResolvedWith({
      table: expect.objectContaining({ table_id: 2 }),
      fields: expect.any(Array),
      entries: expect.any(Array),
      children: expect.any(Array)
    });
  });

  it("handles table creation error", async ({ worker }) => {
    worker.use(http.post('https://www.example.com/api/tables', () => {
      return HttpResponse.json("A server error occured.", { status: 500 })
    }));

    const loadResult = await load({ params: {} })
    const data = $state(loadResult)
    const screen = page.render(Layout, { props: { children: mockChild, data } });
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 1" })).toBeVisible();
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 2" })).toBeVisible();

    await screen.getByRole("checkbox", { name: "add table" }).click();

    const createInput = screen.getByRole("textbox", { name: "table name" });
    await expect.element(createInput).toBeVisible();
    await createInput.fill("My New Table");

    await screen.getByRole("button", { name: "create" }).click()

    await expect.element(screen.getByRole('paragraph').filter({ hasText: new RegExp("Error: A server error occured\\.") })).toBeVisible();
  });

  it("handles table import error", async ({ worker }) => {
    worker.use(http.post('https://www.example.com/api/tables/csv', () => {
      return HttpResponse.json("A server error occured.", { status: 500 })
    }));

    let testData = $state({
      tables: [
        {
          access_role: "Owner",
          table: {
            table_id: 123,
            user_id: 456,
            name: "Test Table 1",
            description: "Description"
          }
        },
        {
          access_role: "Owner",
          table: {
            table_id: 124,
            user_id: 456,
            name: "Test Table 2",
            description: "Description"
          }
        }
      ]
    });

    const screen = page.render(Layout, { props: { children: mockChild, data: testData } });
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 1" })).toBeVisible();
    await expect.element(screen.getByRole("listitem").filter({ hasText: "Test Table 2" })).toBeVisible();

    await screen.getByRole("checkbox", { name: "add table" }).click();

    const fileInput = screen.getByLabelText("file input");
    const fileImportButton = screen.getByRole("button", { name: "import" });

    await expect.element(fileInput).toBeVisible();
    await expect.element(fileImportButton).toBeVisible();

    const testFile = new File(["This, is, a, test, file"], "test.csv");

    await userEvent.upload(fileInput, testFile);
    await fileImportButton.click();

    await expect.element(screen.getByRole('paragraph').filter({ hasText: new RegExp("Error: A server error occured\\.") })).toBeVisible();
  });

});




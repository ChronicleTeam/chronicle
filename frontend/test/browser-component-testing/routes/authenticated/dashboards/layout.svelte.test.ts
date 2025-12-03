// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { expect, vi, afterEach, describe } from 'vitest'
import { createRawSnippet } from 'svelte';
import { page, userEvent } from '@vitest/browser/context'

import Layout from '../../../../../src/routes/(authGuarded)/dashboards/+layout.svelte'
import { load } from '../../../../../src/routes/(authGuarded)/dashboards/+layout.ts'
import { it } from '../../../test-extensions';
import { deleteEntry, patchEntry, postEntries, postExportTable } from '../../../../../src/lib/api/dataManagement.ts';
import { HttpResponse, http } from 'msw';
import { FieldType } from '../../../../../src/lib/types/dataManagement.ts';

const mockChild = createRawSnippet(() => ({
  render: () => `<h1>This is a child component</h1>`,
}));

describe("main dashboard layout", () => {
  it("renders dashboard list", async () => {
    const loadResult = await load()
    const data = $state(loadResult);
    const screen = page.render(Layout, { props: { children: mockChild, data } });

    await expect.element(screen.getByText("Projects")).toBeVisible();
  })
})

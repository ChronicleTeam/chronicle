// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { expect, vi, afterEach, describe } from 'vitest'
import { page, userEvent } from '@vitest/browser/context'

import Page from '../../../../../../src/routes/(authGuarded)/dashboards/[dashboard_id]/+page.svelte'
import { load } from '../../../../../../src/routes/(authGuarded)/dashboards/[dashboard_id]/+page.ts'
import { it } from '../../../../test-extensions';
import { deleteEntry, patchEntry, postEntries, postExportTable } from '../../../../../../src/lib/api/dataManagement.ts';
import { HttpResponse, http } from 'msw';
import { FieldType } from '../../../../../../src/lib/types/dataManagement.ts';

const pageParams = {
  dashboard_id: "1"
}

vi.mock('../../../../../src/lib/api/dataManagement.ts', { spy: true })
vi.mock("$app/state", () => ({
  page: {
    params: pageParams
  }
}));

describe("main dashboard editor", () => {
  it("renders dashboard", async () => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    await expect.element(screen.getByText("Projects")).toBeVisible()
    await expect.element(screen.getByText("Budgets")).toBeVisible()
  })
})

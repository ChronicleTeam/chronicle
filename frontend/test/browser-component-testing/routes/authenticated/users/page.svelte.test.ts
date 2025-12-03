// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { expect, vi, afterEach, describe } from 'vitest'
import { page, userEvent } from '@vitest/browser/context'

import Page from '../../../../../src/routes/users/+page.svelte'
import { it } from '../../../test-extensions';

const pageParams = {
  dashboard_id: "1"
}

vi.mock('../../../../../src/lib/api/dataManagement.ts', { spy: true })
vi.mock("$app/state", () => ({
  page: {
    params: pageParams
  }
}));

describe("user management", () => {
  it("renders management board", async () => {
    const screen = page.render(Page);

    await expect.element(screen.getByText("test@example.com")).toBeVisible()
  })
})

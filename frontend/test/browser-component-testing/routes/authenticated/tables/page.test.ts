// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { expect, vi, afterEach, describe } from 'vitest'
import { page } from '@vitest/browser/context'

import Page from '../../../../../src/routes/(authGuarded)/tables/+page.svelte'
import { it } from '../../../test-extensions';

describe("main table default page", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  })

  it("is rendered", async () => {
    const screen = page.render(Page);
    await expect.element(screen.getByRole("heading", { name: "Select a Table", exact: true })).toBeVisible();
  })
});

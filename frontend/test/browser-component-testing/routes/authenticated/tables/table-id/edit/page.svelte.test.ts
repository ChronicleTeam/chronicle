// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { expect, vi, afterEach, describe } from 'vitest'
import { page, userEvent } from '@vitest/browser/context'

import Page from '../../../../../../../src/routes/(authGuarded)/tables/[table_id]/edit/+page.svelte'
import { load } from '../../../../../../../src/routes/(authGuarded)/tables/[table_id]/edit/+page.ts'
import { it } from '../../../../../test-extensions';
import { deleteEntry, deleteField, getTableData, patchEntry, postEntries, postExportTable, postField } from '../../../../../../../src/lib/api/dataManagement.ts';
import { HttpResponse, http } from 'msw';
import { FieldType } from '../../../../../../../src/lib/types/dataManagement.ts';
// @ts-ignore
import { goto } from '$app/navigation';

const pageParams = {
  table_id: "124"
}
vi.mock('../../../../../src/lib/api/dataManagement.ts', { spy: true })
vi.mock("$app/state", () => ({
  page: {
    params: pageParams
  }
}));

const fieldTypes = [
  ["Text", "Text"],
  ["Integer", "Integer"],
  ["Decimal", "Float"],
  ["Money", "Money"],
  ["Progress", "Progress"],
  ["Date Time", "DateTime"],
  ["WebLink", "WebLink"],
  ["Checkbox", "Checkbox"],
  ["Enumeration", "Enumeration"],
]

describe("main field editor", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  });

  it("renders fields", async () => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });
    await expect.poll(() => getTableData).toHaveBeenCalledTimes(2);

    await expect.element(screen.getByRole("textbox", { name: "Name" })).toHaveValue("Test Table");
    await expect.poll(() => screen.getByRole("textbox", { name: "field name" }).elements().length).toEqual(2);
    await expect.poll(() => screen.getByRole("textbox", { name: "subtable name" }).elements().length).toEqual(1);
  });

  it.for(fieldTypes)("allows for %s field creation", async ([newDisplayType, newType]) => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });
    await expect.poll(() => getTableData).toHaveBeenCalledTimes(2);

    await screen.getByRole("button", { name: "add field" }).click()

    const newFieldTypeSelect = screen.getByRole("combobox", { name: "type" }).last();
    await expect.element(newFieldTypeSelect).toBeVisible();
    await newFieldTypeSelect.selectOptions([newDisplayType]);
    await expect.element(newFieldTypeSelect).toHaveValue(newType);

    await screen.getByRole("button", { name: "save" }).click();
    await screen.getByRole("button", { name: "confirm" }).click();

    await expect.poll(() => postField).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({
      name: "New Field 1",
      table_id: 124,
      field_kind: expect.objectContaining({
        type: newType,
      })
    }))
  });

  it("allows for field deletion", async () => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });
    await expect.poll(() => getTableData).toHaveBeenCalledTimes(2);

    await screen.getByRole("button", { name: "remove" }).first().click();

    await screen.getByRole("button", { name: "save" }).click();
    await screen.getByRole("button", { name: "confirm" }).click();

    await expect.poll(() => deleteField).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({
      table_id: 124,
      field_id: 1,
    }));
  })
})

// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { expect, vi, afterEach, describe } from 'vitest'
import { page, userEvent } from '@vitest/browser/context'

import Page from '../../../../../../src/routes/(authGuarded)/tables/[table_id]/+page.svelte'
import { load } from '../../../../../../src/routes/(authGuarded)/tables/[table_id]/+page.ts'
import { it } from '../../../../test-extensions';
import { patchEntry } from '../../../../../../src/lib/api/dataManagement.ts';
import { HttpResponse, http } from 'msw';

const pageParams = {
  table_id: "124"
}

vi.mock('../../../../../src/lib/api/dataManagement.ts', { spy: true })
vi.mock("$app/state", () => ({
  page: {
    params: pageParams
  }
}));

const createTableDataResponse = (field, cell1, cell2) => {
  return http.get('https://www.example.com/api/tables/:table_id/data', ({ params }) => {
    const table_id = parseInt(params.table_id as string);
    const user_id = 456;
    const startDateStr = "2009-02-12T23:31:30.456Z"
    const endDateStr = "2009-02-14T23:31:30.456Z"
    const date1Str = "2009-02-13T23:31:30.456Z"
    const date2Str = "2009-02-14T23:31:30.123Z"
    return HttpResponse.json({
      table: {
        table_id,
        user_id,
        name: "Test Table",
        description: "Description"
      },
      fields: [
        {
          table_id,
          user_id,
          field_id: 1,
          name: "Text Column",
          ordering: 1,
          field_kind: {
            type: FieldType.Text as FieldType.Text,
            is_required: false,
          }
        },
        field
      ],
      entries: [
        {
          entry_id: 0,
          cells: {
            '1': "test",
            '2': cell1,
          },
        },
        {
          entry_id: 1,
          cells: {
            '1': "test 2",
            '2': cell2,
          },
        },
      ],
      children: []
    });
  }
  )


};

describe("main table editor", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  })

  it("renders table", async () => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    await expect.element(screen.getByRole("heading", { name: "Test Table", exact: true })).toBeVisible();

    await expect.element(screen.getByRole("cell", { name: "Text Column", exact: true })).toBeVisible();
    await expect.element(screen.getByRole("cell", { name: "Date Column", exact: true })).toBeVisible();
    await expect.element(screen.getByRole("cell", { name: "test", exact: true })).toBeVisible();
    await expect.element(screen.getByRole("cell", { name: "test 2", exact: true })).toBeVisible();
    await expect.element(screen.getByRole("cell", { name: "2009-02-13" })).toBeVisible();
    await expect.element(screen.getByRole("cell", { name: "2009-02-14" })).toBeVisible();
  })

  it("allows for editing", async () => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    await screen.getByRole("cell", { name: "test", exact: true }).dblClick();
    await screen.getByRole("textbox", { disabled: false }).first().fill("test 3");

    await screen.getByRole("button", { name: "save" }).click();
    await expect.element(screen.getByRole("button", { name: "save" })).not.toBeInTheDocument();

    await expect.poll(() => screen.getByRole("textbox", { disabled: false }).elements().length).toBe(0);

    await expect.poll(() => patchEntry).toHaveBeenCalledExactlyOnceWith(
      expect.any(Object),
      {
        entry_id: 0,
        cells: {
          '1': 'test 3',
          '2': expect.any(Date)
        }
      })
  })

});





// @ts-ignore
/// <reference types="vitest-browser-svelte" />
import { expect, vi, afterEach, describe } from 'vitest'
import { page, userEvent } from '@vitest/browser/context'

import Page from '../../../../../../src/routes/(authGuarded)/tables/[table_id]/+page.svelte'
import { load } from '../../../../../../src/routes/(authGuarded)/tables/[table_id]/+page.ts'
import { it } from '../../../../test-extensions';
import { deleteEntry, patchEntry, postEntries, postExportTable } from '../../../../../../src/lib/api/dataManagement.ts';
import { HttpResponse, http } from 'msw';
import { FieldType } from '../../../../../../src/lib/types/dataManagement.ts';
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
vi.stubGlobal('open', vi.fn())

const allFieldTableResponse = () => {
  return http.get('https://www.example.com/api/tables/:table_id/data', ({ params }) => {
    const table_id = parseInt(params.table_id as string);
    const user_id = 456;
    const startDateStr = "2009-02-12T23:31:30.456Z"
    const endDateStr = "2009-02-14T23:31:30.456Z"
    const date1Str = "2009-02-13T23:31:30.456Z"
    const date2Str = "2009-02-14T23:31:30.123Z"
    return HttpResponse.json({
      access_role: "Owner",
      table_data: {
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
              type: FieldType.Text,
              is_required: false,
            }
          },
          {
            table_id,
            user_id,
            field_id: 2,
            name: "Integer Column",
            ordering: 2,
            field_kind: {
              type: FieldType.Integer,
              is_required: false,
            }
          },
          {
            table_id,
            user_id,
            field_id: 3,
            name: "Decimal Column",
            ordering: 3,
            field_kind: {
              type: FieldType.Decimal,
              is_required: false,
              scientific_notation: true,
            }
          },
          {
            table_id,
            user_id,
            field_id: 4,
            name: "Money Column",
            ordering: 4,
            field_kind: {
              type: FieldType.Money,
              is_required: false,
            }
          },
          {
            table_id,
            user_id,
            field_id: 5,
            name: "Progress Column",
            ordering: 5,
            field_kind: {
              type: FieldType.Progress,
              total_steps: 10,
            }
          },
          {
            table_id,
            user_id,
            field_id: 6,
            name: "DateTime Column",
            ordering: 6,
            field_kind: {
              type: FieldType.DateTime,
              is_required: false,
              date_time_format: "YYYY-MM-DD"
            }
          },
          {
            table_id,
            user_id,
            field_id: 7,
            name: "Weblink Column",
            ordering: 7,
            field_kind: {
              type: FieldType.WebLink,
              is_required: false,
            }
          },
          {
            table_id,
            user_id,
            field_id: 8,
            name: "Checkbox Column",
            ordering: 8,
            field_kind: {
              type: FieldType.Checkbox,
            }
          },
          {
            table_id,
            user_id,
            field_id: 9,
            name: "Enumeration Column",
            ordering: 9,
            field_kind: {
              type: FieldType.Enumeration,
              values: {
                1: "My Val 1",
                2: "My Val 2",
              },
              default_value: 1,
              is_required: false,
            }
          },
          {
            table_id,
            user_id,
            field_id: 10,
            name: "Extended Money Column",
            ordering: 10,
            field_kind: {
              type: FieldType.Money,
              is_required: false,
              range_start: "1.00",
              range_end: "10.00"
            }
          },
        ],
        entries: [],
        children: []
      }
    });
  })
}

describe("main table editor", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  });

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
  });

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
      });
  });

  it("handles editing error", async ({ worker }) => {
    worker.use(http.patch("https://www.example.com/api/tables/:table_id/entries/:entry_id", async () => {
      return HttpResponse.json(" 1: 'Wrong value' ", {
        status: 422
      })
    }));
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });
    await screen.getByRole("cell", { name: "test", exact: true }).dblClick();
    await screen.getByRole("textbox", { disabled: false }).first().fill("test 3");

    await screen.getByRole("button", { name: "save" }).click();
    await expect.element(screen.getByRole("button", { name: "save" })).toBeVisible();

    await expect.poll(() => patchEntry).toHaveBeenCalledExactlyOnceWith(
      expect.any(Object),
      {
        entry_id: 0,
        cells: {
          '1': 'test 3',
          '2': expect.any(Date)
        }
      });

    await expect.element(screen.getByText("Wrong value")).toBeVisible();
  });

  it("allows for row creation", async ({ worker }) => {
    worker.use(allFieldTableResponse());
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    const addBtn = screen.getByRole("button", { name: "add row" });
    await expect.element(addBtn).toBeVisible();
    await addBtn.click();
    await addBtn.click();

    const newRow = screen.getByRole("row").last();
    await expect.element(newRow).toBeVisible();

    const textInput = newRow.getByRole("textbox").nth(0);
    await expect.element(textInput).toBeVisible();
    await textInput.fill("New Value");

    const integerInput = newRow.getByRole("spinbutton").nth(0);
    await expect.element(integerInput).toBeVisible();
    await integerInput.fill("1");

    const decimalInput = newRow.getByRole("spinbutton").nth(1);
    await expect.element(decimalInput).toBeVisible();
    await decimalInput.fill("2.0");

    const moneyInput = newRow.getByRole("spinbutton").nth(2);
    await expect.element(moneyInput).toBeVisible();
    await moneyInput.fill("3.0");

    const progressInput = newRow.getByRole("spinbutton").nth(3);
    await expect.element(progressInput).toBeVisible();
    await progressInput.fill("4");

    const dateTimeInput = newRow.getByRole("textbox").nth(1);
    await expect.element(dateTimeInput).toBeVisible();
    await dateTimeInput.fill(new Date("2025-12-01").toISOString().substring(0, 16));

    const weblinkInput = newRow.getByRole("textbox").nth(2);
    await expect.element(weblinkInput).toBeVisible();
    await weblinkInput.fill("www.example.com");

    const checkboxInput = newRow.getByRole("checkbox").nth(0);
    await expect.element(checkboxInput).toBeVisible();
    await checkboxInput.click()

    const enumInput = newRow.getByRole("combobox").nth(0);
    await expect.element(enumInput).toBeVisible();
    await enumInput.selectOptions(enumInput.getByRole("option", { name: "My Val 2" }))

    const saveBtn = screen.getByRole("button", { name: "save" });
    await expect.element(saveBtn).toBeVisible();
    await saveBtn.click();

    await expect.poll(() => postEntries).toHaveBeenCalledOnce();

    await expect.element(saveBtn).not.toBeInTheDocument();
  });

  it("correctly interprets the enter key", async () => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    const addBtn = screen.getByRole("button", { name: "add row" });
    await expect.element(addBtn).toBeVisible();
    await addBtn.click();

    expect(screen.getByRole("row").elements().length).toEqual(4)
    const newRow = screen.getByRole("row").last();
    await expect.element(newRow).toBeVisible();
    await newRow.getByRole("textbox").first().click()
    await userEvent.keyboard("{Enter}{Enter}");
    expect(screen.getByRole("row").elements().length).toEqual(5)


  })

  it("handles row creation error", async ({ worker }) => {
    worker.use(http.post("https://www.example.com/api/tables/:table_id/entries", async () => {
      return HttpResponse.json(" 1: 'Wrong value' ", {
        status: 422
      })
    }));
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    const addBtn = screen.getByRole("button", { name: "add row" });
    await expect.element(addBtn).toBeVisible();
    await addBtn.click();

    const saveBtn = screen.getByRole("button", { name: "save" });
    await expect.element(saveBtn).toBeVisible();
    await saveBtn.click();

    await expect.poll(() => postEntries).toHaveBeenCalledOnce();
    await expect.element(screen.getByText("Wrong value")).toBeVisible();
  });

  it("allows for row deletion", async () => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    await screen.getByRole("cell", { name: "test", exact: true }).dblClick();
    await screen.getByRole("button", { name: "delete entry" }).click();
    await screen.getByRole("button", { name: "confirm delete" }).click();

    await expect.poll(() => deleteEntry).toHaveBeenCalledOnce();
    data.table.entries.splice(0, 1);

    await expect.element(screen.getByRole("button", { name: "delete entry" })).not.toBeInTheDocument();
    await expect.element(screen.getByRole("cell", { name: "test", exact: true })).not.toBeInTheDocument();
  });

  it.for(["excel", "csv"])("allows for table %s export", async (fileType) => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    const exportBtn = screen.getByRole("button", { name: "export" });
    await expect.element(exportBtn).toBeVisible();
    await exportBtn.click();

    const fileTypeBtn = screen.getByRole("button", { name: `export as ${fileType}` });
    await expect.element(fileTypeBtn).toBeVisible();
    await fileTypeBtn.click();

    // @ts-ignore
    await expect.poll(() => open.mock.calls[0]).toBeDefined();
    // @ts-ignore
    const blobUrl = open.mock.calls[0]
    const blob = await fetch(blobUrl).then((r) => r.blob());
    await expect(blob.text()).resolves.toEqual(`${fileType} file`);


  })

  it("handles export error", async ({ worker }) => {
    worker.use(http.post('https://www.example.com/api/tables/:table_id/excel', async () => {
      return new HttpResponse("server error", { status: 500 })
    }));

    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    const exportBtn = screen.getByRole("button", { name: "export" });
    await expect.element(exportBtn).toBeVisible();
    await exportBtn.click();

    const excelBtn = screen.getByRole("button", { name: `export as excel` });
    await expect.element(excelBtn).toBeVisible();
    await excelBtn.click();

    // @ts-ignore
    await expect.poll(() => open).not.toHaveBeenCalled();

    await expect.element(screen.getByText("Could not export")).toBeVisible();
  })

  it("Sends to Field Editor", async () => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    await screen.getByRole("button", { name: "Edit" }).first().click()

    await expect.poll(() => goto).toHaveBeenCalledExactlyOnceWith('/tables/124/edit');

  });

  it("sends to Subtable Editor", async ({ worker }) => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    await screen.getByRole("button", { name: "Edit" }).last().click();

    await expect.poll(() => goto).toHaveBeenCalledExactlyOnceWith('/tables/124/subtables/501/edit');
  });
});

describe("access management modal", () => {
  afterEach(async () => {
    vi.clearAllMocks();
  });

  it.for(["Owner", "Editor", "Viewer"])("allows for user addition of role %s", async (role) => {
    const loadResult = await load({ params: pageParams })
    const data = $state(loadResult);
    const screen = page.render(Page, { props: { data } });

    const shareBtn = screen.getByRole("button", { name: "share" });
    await shareBtn.click();

    const usernameField = screen.getByTitle("username");
    await expect.element(usernameField).toBeVisible();
    const roleSelect = screen.getByTitle("role");
    await expect.element(roleSelect).toBeVisible();
    const addUserBtn = screen.getByText("Add", { exact: true });
    await expect.element(addUserBtn).toBeVisible();

    await usernameField.fill("test2@example.com");
    await roleSelect.selectOptions(
      roleSelect.getByRole("option", { name: role })
    );
    await addUserBtn.click();
    data.allAccess.push({
      access_role: role,
      username: "test2@example.com",
    });

    await expect.element(screen.getByText("test2@example.com")).toBeVisible()
  });

});






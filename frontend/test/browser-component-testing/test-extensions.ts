import { test as baseTest } from 'vitest'
import { http, HttpResponse } from 'msw'
import { setupWorker } from 'msw/browser'
import { type SetupWorker } from 'msw/browser'
import { FieldType } from '../../src/lib/types'

const handlers = [
  http.get('https://www.example.com/api/user', () => {
    return HttpResponse.json({
      user_id: 1,
      username: 'test',
    })
  }),
  http.post('https://www.example.com/api/login', async ({ request }) => {
    const formData = await request.formData()
    if (formData.get("username") === "test@example.com" && formData.get("password") === "test123") {
      return HttpResponse.json({
        user_id: 1,
        username: 'test',
      })
    } else {
      return new HttpResponse("Invalid Credentials", {
        status: 422
      })
    }
  }),
  http.get('https://www.example.com/api/logout', () => {
    return new HttpResponse(null, { status: 200 });
  }),
  http.post('https://www.example.com/api/tables', () => {
    return HttpResponse.json({
      table_id: 1,
      user_id: 456,
      name: "A New Table",
      description: "a description"
    })
  }),
  http.post('https://www.example.com/api/tables/csv', () => {
    return HttpResponse.json({
      table: {
        table_id: 2,
        user_id: 456,
        name: "A New Table",
        description: "a description"
      },
      fields: [],
      entries: [],
      children: [],
    })
  }),
  http.get('https://www.example.com/api/tables', () => {
    return HttpResponse.json([
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
    ]);
  }),
  http.get('https://www.example.com/api/tables/:table_id/data', ({ params }) => {
    // if < 500 return normal table. else return a subtable
    if (parseInt(params.table_id as string) < 500) {
      const table_id = parseInt(params.table_id as string);
      const user_id = 456;
      const startDateStr = "2009-02-12T23:31:30.456Z"
      const endDateStr = "2009-02-14T23:31:30.456Z"
      const date1Str = "2009-02-13T23:31:30.456Z"
      const date2Str = "2009-02-14T23:31:30.123Z"
      return HttpResponse.json(
        {
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
                  type: FieldType.Text as FieldType.Text,
                  is_required: false,
                }
              },
              {
                table_id,
                user_id,
                field_id: 2,
                name: "Date Column",
                ordering: 2,
                field_kind: {
                  type: FieldType.DateTime as FieldType.DateTime,
                  is_required: false,
                  range_start: startDateStr,
                  range_end: endDateStr,
                  date_time_format: 'N/A',
                }
              },
            ],
            entries: [
              {
                entry_id: 0,
                cells: {
                  '1': "test",
                  '2': date1Str,
                },
              },
              {
                entry_id: 1,
                cells: {
                  '1': "test 2",
                  '2': date2Str,
                },
              },
            ],
            children: [{
              table: {
                table_id: 501,
                user_id,
                parent_id: table_id,
                name: "Subtable 1",
                description: "",
              },
              fields: [],
              entries: [],
              children: [],
            }]
          }
        }
      );
    } else {
      return HttpResponse.json(
        {
          access_role: "Owner",
          table_data: {
            table: {
              parent_id: 124,
              table_id: parseInt(params.table_id as string),
              user_id: 456,
              name: "Test Subtable",
              description: "Description"
            },
            entries: [],
            fields: [],
            children: [],
          }
        }
      )
    }
  }),
  http.patch('https://www.example.com/api/tables/:table_id/entries/:entry_id', () => {
    return new HttpResponse()
  }),
  http.delete('https://www.example.com/api/tables/:table_id/entries/:entry_id', () => {
    return new HttpResponse()
  }),
  http.get('https://www.example.com/api/users', () => {
    return HttpResponse.json([
      {
        is_admin: true,
        user_id: 456,
        username: "test@example.com",
      },
      {
        is_admin: false,
        user_id: 457,
        username: "test2@example.com",
      },
    ]);
  }),
  http.get('https://www.example.com/api/:rsrc/:rsrc_id/access', () => {
    return HttpResponse.json([
      {
        access_role: "Owner",
        username: "test@example.com",
      },
    ]);
  }),
  http.post('https://www.example.com/api/Table/:table_id/access', () => {
    return new HttpResponse();
  }),
  http.post('https://www.example.com/api/tables/:table_id/entries', async ({ request }) => {
    return HttpResponse.json(await request.json());
  }),
  http.post('https://www.example.com/api/tables/:table_id/excel', async () => {
    return HttpResponse.arrayBuffer(new TextEncoder().encode("excel file").buffer, {
      headers: {
        'Content-Type': 'application/octet-stream',
      }
    });
  }),
  http.post('https://www.example.com/api/tables/:table_id/csv', async () => {
    return HttpResponse.arrayBuffer(new TextEncoder().encode("csv file").buffer, {
      headers: {
        'Content-Type': 'application/octet-stream',
      }
    });
  }),
  http.post('https://www.example.com/api/tables/:table_id/fields', async ({ request }) => {
    return HttpResponse.json(await request.json());
  }),
  http.delete('https://www.example.com/api/tables/:table_id/fields/:field_id', async () => {
    return new HttpResponse();
  }),
  http.get('https://www.example.com/api/dashboards', async () => {
    return HttpResponse.json([
      {
        "dashboard": {
          "dashboard_id": 1,
          "name": "Projects",
          "description": "",
          "created_at": "2025-11-22T02:20:53.723142Z",
          "updated_at": "2025-12-02T20:14:56.263469Z"
        },
        "access_role": "Owner"
      }
    ]);
  }),
  http.get('https://www.example.com/api/dashboards/:dash_id/charts', async () => {
    return HttpResponse.json([
      {
        "chart_id": 1,
        "dashboard_id": 1,
        "table_id": 4,
        "name": "Budgets",
        "chart_kind": "Bar",
        "created_at": "2025-12-02T20:10:12.203760Z",
        "updated_at": null
      }
    ])
  }),
  http.get('https://www.example.com/api/dashboards/:dash_id/charts/:chart_id/data', async () => {
    return HttpResponse.json(
      {
        "chart": {
          "chart_id": 1,
          "dashboard_id": 1,
          "table_id": 4,
          "name": "Budgets",
          "chart_kind": "Bar",
          "created_at": "2025-12-02T20:10:12.203760Z",
          "updated_at": null
        },
        "axes": [
          {
            "axis": {
              "axis_id": 1,
              "chart_id": 1,
              "field_id": 11,
              "axis_kind": "X",
              "aggregate": null,
              "created_at": "2025-12-02T20:14:46.704031Z",
              "updated_at": null
            },
            "field_name": "Name",
            "field_kind": {
              "type": "Text",
              "is_required": true
            }
          },
          {
            "axis": {
              "axis_id": 2,
              "chart_id": 1,
              "field_id": 12,
              "axis_kind": "Y",
              "aggregate": null,
              "created_at": "2025-12-02T20:14:46.704031Z",
              "updated_at": null
            },
            "field_name": "Budget",
            "field_kind": {
              "type": "Money",
              "is_required": true,
              "range_start": "0.00",
              "range_end": null
            }
          }
        ],
        "cells": [
          {
            "1": "Project Gamma",
            "2": "25000.0000"
          },
          {
            "1": "Project Beta",
            "2": "60000.0000"
          },
          {
            "1": "Project Alpha",
            "2": "50000.0000"
          }
        ]
      }
    )
  }),
]
const worker = setupWorker(...handlers)

interface Fixtures {
  authenticated: Boolean;
  worker: SetupWorker;
}

export const it = baseTest.extend<Fixtures>({
  authenticated: true,
  worker: [
    async ({ authenticated }, use) => {
      if (!authenticated) {
        worker.use(
          http.get('https://www.example.com/api/user', () => {
            return HttpResponse.json(null)
          })
        );
      } else {
      }
      await worker.start({ quiet: true });

      await use(worker);

      worker.resetHandlers();
      worker.stop();
    },
    {
      auto: true,
    }
  ],
})

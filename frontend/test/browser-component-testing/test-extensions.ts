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
        children: []
      });
    } else {
      return HttpResponse.json(
        {
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
      )
    }
  }),
  http.patch('https://www.example.com/api/tables/:table_id/entries/:entry_id', () => {
    return new HttpResponse()
  })
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

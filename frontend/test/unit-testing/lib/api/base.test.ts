// @ts-ignore
import { goto } from "$app/navigation";
import { vi, describe, beforeEach, it, expect } from "vitest"
import { GET, POST, POST_FORM, PUT, PATCH, DELETE, _TESTING, hydrateJSONTableData } from '../../../../src/lib/api/base'
import { clearUser } from "../../../../src/lib/user.svelte.js";
import { FieldType, type DateTimeKind } from "../../../../src/lib/types/dataManagement.js";

vi.mock('$lib/user.svelte.js', () => ({
  clearUser: vi.fn()
}));
vi.mock('$app/navigation', () => ({
  goto: vi.fn()
}));
vi.stubGlobal('fetch', vi.fn(() => new Promise((res) => res(Response.json({ test: 'hello' })))));

const handleResponse = _TESTING.handleResponse;

describe('Base API functions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('GET', () => {
    it('Should fetch the right endpoint', async () => {
      const result = await GET('/testGET');

      expect(result).toEqual({ test: 'hello' });
      expect(fetch).toHaveBeenCalledWith(
        'example.com/api/testGET',
        expect.objectContaining({
          method: 'GET'
        })
      );
    });
  });

  describe('POST', () => {
    it('Should fetch the right endpoint with the right body', async () => {
      const body = {
        testBody: 123,
      };

      const result = await POST('/testPOST', body);

      expect(result).toEqual({ test: 'hello' });
      expect(fetch).toHaveBeenCalledWith(
        'example.com/api/testPOST',
        expect.objectContaining({
          method: 'POST',
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body),
        })
      );
    });
  });

  describe('POST_FORM', () => {
    it('Should fetch the right endpoint with the right non-blob form data', async () => {
      const body = new FormData();
      body.append('testField', '123');

      const result = await POST_FORM('/testPOST_FORM', body);

      expect(result).toEqual({ test: 'hello' });
      expect(fetch).toHaveBeenCalledWith(
        'example.com/api/testPOST_FORM',
        expect.objectContaining({
          method: 'POST',
          body: expect.any(URLSearchParams),
        })
      );
      // @ts-ignore fetch will be mocked
      expect([...fetch.mock.lastCall[1].body.entries()]).toEqual([['testField', '123']]);
    });

    it('Should fetch the right endpoint with the right blob form data', async () => {
      const blob = new Blob(["this is a blob"]);
      const blobBytes = await blob.bytes();
      const body = new FormData();
      body.append('testField', '123');
      body.append('testBlob', blob);

      const result = await POST_FORM('/testPOST_FORM', body);

      expect(result).toEqual({ test: 'hello' });
      expect(fetch).toHaveBeenCalledWith(
        'example.com/api/testPOST_FORM',
        expect.objectContaining({
          method: 'POST',
          body: expect.any(FormData),
        })
      );
      // @ts-ignore fetch will be mocked
      expect([...fetch.mock.lastCall[1].body.entries()]).toEqual([
        ['testField', '123'],
        ['testBlob', expect.any(Blob)]
      ]);
      // @ts-ignore fetch will be mocked
      const apiCallBlobBytes = await fetch.mock.lastCall[1].body.get('testBlob').bytes();
      expect(apiCallBlobBytes).toEqual(blobBytes);
    });
  });

  describe('PUT', () => {
    it('Should fetch the right endpoint with the right body', async () => {
      const body = {
        testBody: 456,
      };

      const result = await PUT('/testPUT', body);

      expect(result).toEqual({ test: 'hello' });
      expect(fetch).toHaveBeenCalledWith(
        'example.com/api/testPUT',
        expect.objectContaining({
          method: 'PUT',
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body),
        })
      );
    });
  });


  describe('PATCH', () => {
    it('Should fetch the right endpoint with the right body ', async () => {
      const body = {
        testBody: 789,
      };

      const result = await PATCH('/testPATCH', body);

      expect(result).toEqual({ test: 'hello' });
      expect(fetch).toHaveBeenCalledWith(
        'example.com/api/testPATCH',
        expect.objectContaining({
          method: 'PATCH',
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body),
        })
      );
    });
  });

  describe('DELETE', () => {
    it('Should fetch the right endpoint', async () => {
      const result = await DELETE('/testDELETE');

      expect(result).toBeUndefined;
      expect(fetch).toHaveBeenCalledWith(
        'example.com/api/testDELETE',
        expect.objectContaining({
          method: 'DELETE',
        })
      );
    });

    it('Should throw if response is not ok', async () => {
      const badResponse = new Response("Bad Response", { status: 400, statusText: "This is an error" })
      // @ts-ignore fetch should be mocked
      fetch.mockResolvedValueOnce(badResponse);
      try {
        await DELETE('/testDELETE');
        expect.unreachable("Delete should fail");
      } catch (e) {
        expect(e).toEqual({
          status: 400,
          body: "This is an error",
        });
      }
    })
  });
});

describe('Base helper functions', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("handleResponse", () => {
    it("Should return a JSON by default", async () => {
      const jsonBody = { testBody: "this is a response" };
      const response = Response.json(jsonBody, { status: 200 });

      const result = await handleResponse(response);

      expect(result).toEqual(jsonBody);
    });

    it("Should return a Blob if content-type is octet-stream", async () => {
      const blobBody = new Blob(["This is a response"]);
      const response = new Response(blobBody, { status: 200, headers: { 'Content-Type': 'application/octet-stream' } });

      const result = await handleResponse<Blob>(response);

      const responseBytes = await blobBody.bytes();
      const resultBytes = await result.bytes();
      expect(resultBytes).toEqual(responseBytes);

    });

    it("Should return an empty Object if unparseable", async () => {
      const blobBody = new Blob(["This is a response"]);
      const response = new Response(blobBody, { status: 200, });

      const result = await handleResponse(response);
      expect(result).toEqual({});
    });

    it("Should throw if response not OK (blob text respone)", async () => {
      const blobBody = new Blob(["This is an error"]);
      const response = new Response(blobBody, { status: 400, });

      await expect(handleResponse(response)).rejects.toThrow(
        expect.objectContaining({
          status: 400,
          body: "This is an error"
        })
      );
    })

    it("Should throw if response not OK (json text respone)", async () => {
      const response = Response.json("This is an error", { status: 400, });

      await expect(handleResponse(response)).rejects.toThrow(
        expect.objectContaining({
          status: 400,
          body: "This is an error"
        })
      );
    });

    it("Should throw if response not OK (json Object respone)", async () => {
      const response = Response.json({ message: "This is an error" }, { status: 400, });

      try {
        await handleResponse(response);
        expect.unreachable("handleResponse should throw")
      } catch (e) {
        expect(e).toEqual({
          status: 400,
          body: expect.objectContaining({ message: "This is an error" })
        });
        expect(e.body.toString()).toEqual("message: This is an error");
      }
    })

    it("Should throw if response not OK (fallback to statusText)", async () => {
      const response = new Response("[This is NOT an error", {
        status: 400,
        statusText: "This is an error",
        headers: { "Content-Type": "application/json" },
      });

      await expect(handleResponse(response)).rejects.toThrow(
        expect.objectContaining({
          status: 400,
          body: "This is an error"
        })
      );
    })

    it("Should throw and redirect if response is 401", async () => {
      const blobBody = new Blob(["This is an error"]);
      const response = new Response(blobBody, { status: 401, });

      await expect(handleResponse(response)).rejects.toThrow(
        expect.objectContaining({
          status: 401,
          body: "This is an error"
        })
      );
      expect(clearUser).toHaveBeenCalledOnce();
      expect(goto).toHaveBeenCalledOnce();
    })
  });

  describe("hydrateJSONTableData", () => {
    it("Should turn Table DateTime entries and field ranges to Date Objects", async () => {

      const startDateStr = "2009-02-12T23:31:30.456Z"
      const endDateStr = "2009-02-14T23:31:30.456Z"
      const date1Str = "2009-02-13T23:31:30.456Z"
      const date2Str = "2009-02-13T23:31:30.123Z"

      const table_id = 123;
      const user_id = 456;
      const table = {
        table_id,
        user_id,
        name: "Test Table",
        description: "Description"
      }
      const data = {
        table: table,
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
      };

      const result = hydrateJSONTableData(data);

      expect((result.fields[1].field_kind as DateTimeKind).range_start?.toISOString()).toEqual(startDateStr)
      expect((result.fields[1].field_kind as DateTimeKind).range_end?.toISOString()).toEqual(endDateStr)

      expect(result.entries[0].cells['1']).toEqual("test");
      expect(result.entries[1].cells['1']).toEqual("test 2");

      // @ts-ignore
      expect(result.entries[0].cells['2'].toISOString()).toEqual(date1Str);
      // @ts-ignore
      expect(result.entries[1].cells['2'].toISOString()).toEqual(date2Str);
    });

    it("Should turn Table DateTime entries without ranges to Date Objects", async () => {

      const date1Str = "2009-02-13T23:31:30.456Z"
      const date2Str = "2009-02-13T23:31:30.123Z"

      const table_id = 123;
      const user_id = 456;
      const table = {
        table_id,
        user_id,
        name: "Test Table",
        description: "Description"
      }
      const data = {
        table: table,
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
      };

      const result = hydrateJSONTableData(data);

      expect(result.entries[0].cells['1']).toEqual("test");
      expect(result.entries[1].cells['1']).toEqual("test 2");
      // @ts-ignore
      expect(result.entries[0].cells['2'].toISOString()).toEqual(date1Str);
      // @ts-ignore
      expect(result.entries[1].cells['2'].toISOString()).toEqual(date2Str);
    });
  });
});

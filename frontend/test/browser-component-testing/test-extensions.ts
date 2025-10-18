import { test as baseTest } from 'vitest'
import { http, HttpResponse } from 'msw'
import { setupWorker } from 'msw/browser'
import { type SetupWorker } from 'msw/browser'

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
        const newHandlers = [
          http.get('https://www.example.com/api/user', () => {
            return HttpResponse.json(null)
          }),
          ...handlers.slice(1)
        ]
        worker.resetHandlers(...newHandlers);
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

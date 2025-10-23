import 'vitest-browser-svelte'
import '../../src/app.css'
import { vi, beforeEach } from 'vitest'
import { clearUser } from '../../src/lib/user.svelte.ts'

beforeEach(async () => {
  vi.resetModules();
  await clearUser();
})


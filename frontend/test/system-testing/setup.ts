import { vi } from 'vitest'

vi.mock('$env/dynamic/public', () => ({
  env: { PUBLIC_API_URL: 'localhost:5000' }
}));


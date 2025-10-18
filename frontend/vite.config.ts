/// <reference types="vitest/config" />
/// <reference types="@vitest/browser/providers/playwright" />
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from "@tailwindcss/vite";
import { resolve } from 'node:path';

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  test: {
    coverage: {
      provider: 'v8',
      include: ['src/lib/**', 'src/routes/**'],
    },
    projects: [
      {
        plugins: [tailwindcss(), sveltekit()],
        test: {
          setupFiles: ['test/unit-testing/setup.ts'],
          include: ['test/unit-testing/**/*.test.ts'],
          name: 'unit',
          environment: 'node',
        }
      }, {
        plugins: [tailwindcss(), sveltekit()],
        test: {
          setupFiles: ['vitest-browser-svelte', 'test/browser-component-testing/setup.ts'],
          include: ['test/browser-component-testing/**/*.test.ts'],
          name: 'system',
          browser: {
            enabled: true,
            provider: "playwright",
            headless: true,
            instances: [
              {
                browser: 'chromium',
              },
            ],
          },
          alias: {
            '$env/dynamic/private': resolve("./test/browser-component-testing/mocks/env.ts"),
            '$env/dynamic/public': resolve("./test/browser-component-testing/mocks/env.ts"),
            '$env/static/private': resolve("./test/browser-component-testing/mocks/env.ts"),
            '$env/static/public': resolve("./test/browser-component-testing/mocks/env.ts"),
            '$app/navigation': resolve("./test/browser-component-testing/mocks/navigation.ts"),
          }
        }
      }
    ]
  },
});

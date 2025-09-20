/// <reference types="vitest" />
/// <reference types="@vitest/browser/matchers" />
/// <reference types="@vitest/browser/providers/playwright" />
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  test: {
    coverage: {
      provider: 'v8',
      include: ['src/lib/**'],
      experimentalAstAwareRemapping: true,
    },
    projects: [
      {
        plugins: [tailwindcss(), sveltekit()],
        test: {
          include: ['test/unit-testing/**'],
          name: 'unit',
          environment: 'node',
        }
      }, {
        plugins: [tailwindcss(), svelte()],
        test: {
          include: ['test/system-testing/**/*.test.ts'],
          name: 'system',
          browser: {
            enabled: true,
            provider: 'playwright',
            headless: true,
            instances: [
              { browser: 'chromium' },
              { browser: 'firefox' },
            ],
          },
        }
      }
    ]
  },
});

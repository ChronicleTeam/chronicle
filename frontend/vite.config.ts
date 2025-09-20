/// <reference types="vitest" />
/// <reference types="@vitest/browser/matchers" />
/// <reference types="@vitest/browser/providers/playwright" />
/// <reference types="@vitest/browser/context" />
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
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
        plugins: [tailwindcss(), sveltekit()],
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

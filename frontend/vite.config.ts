/// <reference types="vitest" />
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
  plugins: [sveltekit()],
  test: {
    coverage: {
      provider: 'v8',
      include: ['src/lib/**'],
      experimentalAstAwareRemapping: true,
    },
  },
});

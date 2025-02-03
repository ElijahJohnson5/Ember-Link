import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    alias: {
      '~/': new URL('./src/', import.meta.url).pathname,
      '@ember-link/protocol': new URL('../protocol/src/index.ts', import.meta.url).pathname,
      '@ember-link/protocol/*': new URL('../protocol/src/*', import.meta.url).pathname,
      '@ember-link/event-emitter': new URL('../event-emitter/src/index.ts', import.meta.url)
        .pathname,
      '@ember-link/event-emitter/*': new URL('../event-emitter/src/*', import.meta.url).pathname,
      '@ember-link/storage': new URL('../storage/src/index.ts', import.meta.url).pathname,
      '@ember-link/storage/*': new URL('../storage/src/*', import.meta.url).pathname
    }
  }
});

import { defineWorkspace } from 'vitest/config';

export default defineWorkspace([
  './packages/core/vitest.config.ts',
  './packages/react/vite.config.ts',
  './examples/react-cursors/vite.config.ts'
]);

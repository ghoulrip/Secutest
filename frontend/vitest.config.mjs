import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: './src/setupTests.ts',
    include: ['src/**/*.test.{ts,tsx}'],
    exclude: ['node_modules', 'dist'],
  },
  coverage: {
    provider: 'v8',
    reporter: ['text', 'json', 'html'],
    include: [
      'src/api/client.test.ts',
      'src/pages/ScanPage.test.ts',
    ],
    exclude: [
      'src/**/*.test.{ts,tsx}',
      'src/**/*.spec.{ts,tsx}',
      'src/setupTests.ts',
      'src/main.ts',
    ],
  },
})
import { fileURLToPath } from 'node:url'
import { configDefaults, defineConfig, mergeConfig } from 'vitest/config'
import viteConfig from './vite.config'

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: 'jsdom',
      root: fileURLToPath(new URL('./', import.meta.url)),
      exclude: [...configDefaults.exclude],
      restoreMocks: true,
      coverage: {
        provider: 'v8',
        reporter: ['text-summary', 'lcov', 'cobertura'],
        reportsDirectory: 'coverage',
        include: ['src/**/*.{ts,vue}'],
        exclude: ['src/api/generated.ts', 'src/main.ts', 'src/**/__tests__/**'],
      },
    },
  }),
)

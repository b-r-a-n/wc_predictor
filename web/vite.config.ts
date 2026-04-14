import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import { resolve } from 'path'
import { execSync } from 'child_process'

function resolveCommitSha(): string {
  const fromEnv = process.env.GITHUB_SHA
  if (fromEnv) return fromEnv.slice(0, 7)
  try {
    return execSync('git rev-parse --short HEAD', { cwd: __dirname }).toString().trim()
  } catch {
    return 'dev'
  }
}

const commitSha = resolveCommitSha()

// https://vite.dev/config/
export default defineConfig({
  base: '/wc_predictor/',
  define: {
    __COMMIT_SHA__: JSON.stringify(commitSha),
  },
  plugins: [
    react(),
    tailwindcss(),
    wasm(),
    topLevelAwait(),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  optimizeDeps: {
    exclude: ['wasm-pkg'],
  },
  build: {
    target: 'esnext',
  },
})

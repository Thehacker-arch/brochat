import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  server: {
    host: '0.0.0.0', // Allow external connections
    port: 5173,
    strictPort: true,
    hmr: false,
    cors: true
  },
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
    extensions: ['.mjs', '.js', '.ts', '.jsx', '.tsx', '.json', '.vue']
  },
  esbuild: {
    tsconfigRaw: {
      compilerOptions: {
        importsNotUsedAsValues: "preserve"
      }
    }
  }
})
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { VitePWA } from 'vite-plugin-pwa'

const base = process.env.BASE_PATH || '/'

export default defineConfig({
  base,
  plugins: [
    svelte(),
    VitePWA({
      registerType: 'autoUpdate',
      includeAssets: ['icons/*.png', 'mark.png', 'logo-full.png', 'feed-products.json'],
      manifest: {
        name: 'ทีเด็ดปลาน้ำจืด',
        short_name: 'ทีเด็ดปลา',
        description: 'ผู้ช่วยฟาร์มปลาน้ำจืดประจำวัน: อาหาร น้ำ การโต ต้นทุน กำไร',
        lang: 'th',
        dir: 'ltr',
        start_url: base,
        scope: base,
        display: 'standalone',
        orientation: 'portrait',
        background_color: '#F4F7FB',
        theme_color: '#1B2440',
        categories: ['productivity', 'business'],
        icons: [
          { src: base + 'icons/icon-192.png', sizes: '192x192', type: 'image/png' },
          { src: base + 'icons/icon-512.png', sizes: '512x512', type: 'image/png' },
          { src: base + 'icons/icon-512-maskable.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
        ],
      },
      workbox: {
        // เวอร์ชันใหม่ต้องมีผลทันทีที่เปิดแอปรอบถัดไป ไม่ค้างของเก่า
        skipWaiting: true,
        clientsClaim: true,
        cleanupOutdatedCaches: true,
        globPatterns: ['**/*.{js,css,html,svg,png,wasm,woff2,json}'],
        maximumFileSizeToCacheInBytes: 6 * 1024 * 1024,
        navigateFallback: base + 'index.html',
        navigateFallbackDenylist: [/^\/api\//],
        runtimeCaching: [
          {
            urlPattern: /^https:\/\/fonts\.(googleapis|gstatic)\.com\/.*/i,
            handler: 'CacheFirst',
            options: { cacheName: 'google-fonts', expiration: { maxEntries: 20, maxAgeSeconds: 60 * 60 * 24 * 365 } },
          },
          {
            urlPattern: /\/api\/(species|rules|prices|disease-reports|weather)/,
            handler: 'NetworkFirst',
            options: { cacheName: 'api-reference', networkTimeoutSeconds: 5, expiration: { maxEntries: 50, maxAgeSeconds: 60 * 60 * 24 } },
          },
          {
            urlPattern: /\/api\/(farms|crops|ponds|me)/,
            handler: 'NetworkFirst',
            options: { cacheName: 'api-data', networkTimeoutSeconds: 6, expiration: { maxEntries: 200, maxAgeSeconds: 60 * 60 * 24 * 7 } },
          },
        ],
      },
    }),
  ],
  server: {
    port: 5173,
    proxy: { '/api': { target: 'http://127.0.0.1:8787', changeOrigin: true } },
  },
  build: { target: 'es2022', sourcemap: false },
})

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import Icons from 'unplugin-icons/vite'
import IconsResolver from 'unplugin-icons/resolver'
import { AntDesignVueResolver } from 'unplugin-vue-components/resolvers'
import { fileURLToPath, URL } from "node:url"

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;



// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    AutoImport({
      resolvers: [
        AntDesignVueResolver({ importStyle: false }),
        IconsResolver({ prefix: 'Icon' }),
      ],
    }),
    Components({
      resolvers: [
        AntDesignVueResolver({ importStyle: false }),
        IconsResolver({ enabledCollections: ['ri'] }),
      ],
    }),
    Icons({
      autoInstall: true,
    }),

  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));


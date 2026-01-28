import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
    plugins: [vue()],
    clearScreen: false,
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
            ignored: ["**/src-tauri/**"],
        },
    },
    build: {
        outDir: 'dist',
        rollupOptions: {
            output: {
                assetFileNames: (assetInfo) => {
                    if (assetInfo.name && /\.(png|jpe?g|svg|gif|webp)$/.test(assetInfo.name)) {
                        return 'assets/images/[name]-[hash][extname]';
                    }
                    return 'assets/[ext]/[name]-[hash][extname]';
                }
            }
        }
    }
}));

import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { wasm } from "@rollup/plugin-wasm";

const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  mode: "production",
  plugins: [
    wasm({
      targetEnv: "auto-inline",
      maxFileSize: 100000000,
    }),
  ],
  build: {
    emptyOutDir: true,
    lib: {
      entry: resolve(__dirname, "lib.ts"),
      name: "heliosExEx",
      fileName: "lib",
      formats: ["umd", "es"],
    },
  },
});

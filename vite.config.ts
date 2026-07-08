import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  root: "examples/app",
  build: {
    outDir: "../../dist",
    emptyOutDir: true,
  },
});

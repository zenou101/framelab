import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import { defineConfig } from "vite";

// Project pages are served from https://<user>.github.io/framelab/ .
// Override SITE_BASE when the repository is renamed or a custom domain is used.
const base = process.env.SITE_BASE || "/framelab/";

const root = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  base,
  root,
  publicDir: "public",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    assetsInlineLimit: 1024,
    rollupOptions: {
      input: {
        index: resolve(root, "index.html"),
        notFound: resolve(root, "404.html"),
      },
    },
  },
});

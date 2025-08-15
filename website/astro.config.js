import { defineConfig } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  srcDir: ".",
  // Fill this environment variable when building and deploying the site
  base: process.env.ASTRO_CONFIG_BASE,
  vite: {
    plugins: [tailwindcss()],
  },
});

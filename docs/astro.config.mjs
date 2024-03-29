import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import preact from '@astrojs/preact';
import sitemap from '@astrojs/sitemap';
import solidJS from "@astrojs/solid-js";


export default defineConfig({
  integrations: [
    mdx(),
    sitemap(),
    preact({
      compat: true,
      include: ["**/*.tsx"],
      exclude: ["**/*.solid.tsx"]
    }),
    solidJS({
      include: ["**/*.solid.tsx"],
    }),
  ],
  markdown: {
    shikiConfig: {
      experimentalThemes: {
        light: "github-light",
        dark: "github-dark",
      },
    },
  },
  site: "https://shiro.github.io",
  base: "/map2",
  server: { port: 3000 },
});

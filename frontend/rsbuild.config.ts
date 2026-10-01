import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";

// https://rsbuild.dev/guide/basic/configure-rsbuild
export default defineConfig({
  plugins: [pluginReact()],
  output: {
    distPath: {
      root: process.env.WORKERS_FRONTEND_DIST || "dist",
    },
    copy: [{ from: "./src/assets" }],
  },
  html: {
    tags: (tags, { entryName }) =>
      entryName === "index"
        ? [
            ...tags,
            {
              tag: "link",
              attrs: {
                rel: "canonical",
                href: "https://haikunator-generator.goosysapp.net/",
              },
              head: true,
            },
            {
              tag: "link",
              attrs: {
                rel: "alternate",
                type: "text/markdown",
                href: "/index.md",
                title: "Haikunator Generator — Markdown",
              },
              head: true,
            },
          ]
        : tags,
    favicon: "src/assets/favicon.svg",
    title({ entryName }) {
      const titles = {
        404: "404 Not Found",
      };
      return (
        titles[entryName] ||
        "Haikunator Generator | Heroku-like memorable random strings"
      );
    },
    meta: {
      "og:title": { property: "og:title", content: "Haikunator Generator" },
      "og:type": { property: "og:type", content: "website" },
      "og:url": {
        property: "og:url",
        content: "https://haikunator-generator.goosysapp.net/",
      },
      "og:image": {
        property: "og:image",
        content: "https://haikunator-generator.goosysapp.net/ogp.png",
      },
      "og:site_name": {
        property: "og:site_name",
        content: "Haikunator Generator",
      },
      "og:description": {
        property: "og:description",
        content: "Heroku-like memorable random strings",
      },
      "twitter:card": "summary_large_image",
      "twitter:description": "Heroku-like memorable random strings",
      "twitter:title": "Haikunator Generator",
      "twitter:site": "@goosys",
      "twitter:image": "https://haikunator-generator.goosysapp.net/ogp.png",
      "twitter:creator": "@goosys",
      description: "Heroku-like memorable random strings",
    },
  },
  source: {
    entry: {
      index: "./src/index.tsx",
      404: "./src/404.tsx",
    },
  },
  server: {
    host: "0.0.0.0",
    port: 5153,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8787",
        changeOrigin: true,
        secure: false,
      },
      "/mcp": {
        target: "http://127.0.0.1:8787",
        changeOrigin: true,
        secure: false,
      },
    },
  },
});

import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";
import site from "./src/content/site.json";

const siteUrl = new URL("/", site.url);
const ogpUrl = new URL("ogp.png", siteUrl).href;

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
                href: siteUrl.href,
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
        content: siteUrl.href,
      },
      "og:image": {
        property: "og:image",
        content: ogpUrl,
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
      ...(site.twitterSite ? { "twitter:site": site.twitterSite } : {}),
      "twitter:image": ogpUrl,
      ...(site.twitterCreator ? { "twitter:creator": site.twitterCreator } : {}),
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

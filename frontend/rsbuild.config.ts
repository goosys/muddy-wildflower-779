import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";

// https://rsbuild.dev/guide/basic/configure-rsbuild
export default defineConfig({
  plugins: [pluginReact()],
  output: {
    copy: [
      { from: './src/assets' },
    ],
  },
  html: {
    favicon: "src/assets/favicon.ico",
    title({ entryName }) {
      const titles = {
        404: "404 Not Found"
      };
      return titles[entryName] || "Haikunator Generator | Heroku-like memorable random string";
    },
    meta: {
      "og:title": { property: "og:title", content: "Haikunator Generator"},
      "og:type": { property: "og:type", content: "website"},
      "og:url": { property: "og:url", content: "https://haikunator-generator.shuttleapp.rs/"},
      "og:image": { property: "og:image", content: "https://haikunator-generator.shuttleapp.rs/ogp.png"},
      "og:site_name": { property: "og:site_name", content: "Haikunator Generator"},
      "og:description": { property: "og:description", content: "Heroku-like memorable random string"},
      "twitter:card": "summary_large_image",
      "twitter:description": "Heroku-like memorable random string",
      "twitter:title": "Haikunator Generator",
      "twitter:site": "@goosys",
      "twitter:image": "https://haikunator-generator.shuttleapp.rs/ogp.png",
      "twitter:creator": "@goosys",
      description: "Heroku-like memorable random string",
    },
  },
  source: {
    entry: {
      index: "./src/index.tsx",
      404: "./src/404.tsx",
    },
  },
  dev: {
    writeToDisk: true,
  },
  server: {
    port: 5153,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:5150",
        changeOrigin: true,
        secure: false,
      },
    },
  },
});

import type { StorybookConfig } from "@storybook/sveltekit";
import path from "node:path";
import { mergeConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.stories.@(svelte|ts)"],
  addons: ["@storybook/addon-a11y", "@storybook/addon-svelte-csf"],
  framework: {
    name: "@storybook/sveltekit",
    options: {},
  },
  staticDirs: ["../static"],
  viteFinal: async (config) =>
    mergeConfig(config, {
      plugins: [tailwindcss()],
      resolve: {
        alias: {
          $lib: path.resolve(import.meta.url, "../src/lib"),
        },
      },
    }),
};

export default config;

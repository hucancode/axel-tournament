import type { Preview } from "@storybook/svelte";
import "$styles/variables.css";
import "$styles/base.css";
import "$styles/forms.css";
import "$styles/buttons.css";
import "$styles/tabs.css";
import "$styles/dialog.css";

const preview: Preview = {
  parameters: {
    actions: { argTypesRegex: "^on[A-Z].*" },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/,
      },
    },
    layout: "padded",
  },
};

export default preview;

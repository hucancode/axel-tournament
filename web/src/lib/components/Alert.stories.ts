import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import Alert from "$lib/components/Alert.svelte";

const meta = {
  title: "Components/Alert",
  component: Alert,
  args: {
    message: "This is an alert message",
    type: "error",
  },
} satisfies Meta<typeof Alert>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Error: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByText("This is an alert message")).toBeInTheDocument();
  },
};

export const Success: Story = {
  args: {
    message: "Operation completed successfully",
    type: "success",
  },
};

export const Warning: Story = {
  args: {
    message: "Please check your input",
    type: "warning",
  },
};

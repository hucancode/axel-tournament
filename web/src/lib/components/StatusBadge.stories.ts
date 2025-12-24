import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import StatusBadge from "$lib/components/StatusBadge.svelte";

const meta = {
  title: "Components/StatusBadge",
  component: StatusBadge,
  args: {
    status: "running",
    label: "Running",
  },
} satisfies Meta<typeof StatusBadge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Running: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByText("Running")).toBeInTheDocument();
  },
};

export const Cancelled: Story = {
  args: {
    status: "cancelled",
    label: "Cancelled",
  },
};

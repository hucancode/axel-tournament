import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import EmptyState from "$lib/components/EmptyState.svelte";

const meta = {
  title: "Components/EmptyState",
  component: EmptyState,
  args: {
    message: "No items found",
  },
} satisfies Meta<typeof EmptyState>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByText("No items found")).toBeInTheDocument();
  },
};

export const NoTournaments: Story = {
  args: {
    message: "No active tournaments at the moment. Check back soon!",
  },
};

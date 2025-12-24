import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import PlayerCount from "$lib/components/PlayerCount.svelte";

const meta = {
  title: "Components/PlayerCount",
  component: PlayerCount,
  args: {
    current: 5,
    min: 4,
    max: 10,
  },
} satisfies Meta<typeof PlayerCount>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Active: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByText("5 / 10")).toBeInTheDocument();
    await expect(canvas.getByText("Active")).toBeInTheDocument();
  },
};

export const Full: Story = {
  args: {
    current: 10,
    min: 4,
    max: 10,
  },
};

export const NeedMore: Story = {
  args: {
    current: 2,
    min: 4,
    max: 10,
  },
};

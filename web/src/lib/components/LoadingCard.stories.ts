import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import LoadingCard from "$lib/components/LoadingCard.svelte";

const meta = {
  title: "Components/LoadingCard",
  component: LoadingCard,
  args: {
    message: "Loading...",
  },
} satisfies Meta<typeof LoadingCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByText("Loading...")).toBeInTheDocument();
  },
};

export const CustomMessage: Story = {
  args: {
    message: "Loading tournaments...",
  },
};

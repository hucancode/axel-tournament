import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import PageHeader from "$lib/components/PageHeader.svelte";

const meta = {
  title: "Components/PageHeader",
  component: PageHeader,
  args: {
    title: "Welcome to Axel Tournament",
    subtitle: "Compete in coding tournaments, submit your AI bots, and climb the leaderboard",
    centered: false,
  },
} satisfies Meta<typeof PageHeader>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByText("Welcome to Axel Tournament")).toBeInTheDocument();
  },
};

export const Centered: Story = {
  args: {
    title: "Tournaments",
    subtitle: "Browse and join programming tournaments",
    centered: true,
  },
};

export const TitleOnly: Story = {
  args: {
    title: "Profile",
  },
};

import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import FilterSelect from "$lib/components/FilterSelect.svelte";

const statusOptions = [
  { value: "all", label: "All Tournaments" },
  { value: "scheduled", label: "Scheduled" },
  { value: "registration", label: "Registration Open" },
  { value: "running", label: "Running" },
  { value: "completed", label: "Completed" },
];

const meta = {
  title: "Components/FilterSelect",
  component: FilterSelect,
  args: {
    label: "Filter by Status:",
    options: statusOptions,
    value: "all",
    disabled: false,
  },
} satisfies Meta<typeof FilterSelect>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByText("Filter by Status:")).toBeInTheDocument();
  },
};

export const Disabled: Story = {
  args: {
    label: "Filter by Status:",
    options: statusOptions,
    value: "running",
    disabled: true,
  },
};

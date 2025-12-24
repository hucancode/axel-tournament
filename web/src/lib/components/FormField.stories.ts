import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import FormField from "$lib/components/FormField.svelte";

const meta = {
  title: "Components/FormField",
  component: FormField,
  args: {
    label: "Email",
    type: "email",
    id: "email",
    value: "",
    required: true,
  },
} satisfies Meta<typeof FormField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Email: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByLabelText("Email")).toBeInTheDocument();
  },
};

export const Password: Story = {
  args: {
    label: "Password",
    type: "password",
    id: "password",
    helpText: "Minimum 8 characters",
    minlength: 8,
  },
};

export const Optional: Story = {
  args: {
    label: "Country Code",
    type: "text",
    id: "location",
    placeholder: "US, UK, FR, etc.",
    maxlength: 2,
    helpText: "2-letter ISO country code",
    required: false,
  },
};

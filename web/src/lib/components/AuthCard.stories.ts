import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import AuthCard from "$lib/components/AuthCard.svelte";

const meta = {
  title: "Components/AuthCard",
  component: AuthCard,
  args: {
    title: "Login",
    error: "",
    loading: false,
  },
} satisfies Meta<typeof AuthCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    showEmailPassword: true,
    onsubmit: (data: { email: string; password: string }) => console.log('Login:', data),
  },
};

export const WithError: Story = {
  args: {
    title: "Login",
    error: "Invalid credentials",
    showEmailPassword: true,
    onsubmit: (data: { email: string; password: string }) => console.log('Login:', data),
  },
};

export const Loading: Story = {
  args: {
    title: "Login",
    loading: true,
    showEmailPassword: true,
    onsubmit: (data: { email: string; password: string }) => console.log('Login:', data),
  },
};

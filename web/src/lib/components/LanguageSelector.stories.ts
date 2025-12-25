import type { Meta, StoryObj } from '@storybook/svelte';
import LanguageSelector from '$lib/components/LanguageSelector.svelte';

const meta = {
  title: 'Components/LanguageSelector',
  component: LanguageSelector,
  parameters: {
    layout: 'padded',
  },
} satisfies Meta<typeof LanguageSelector>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    languages: ['rust', 'go', 'c'],
    selected: ['rust'],
    onToggle: (lang: string) => console.log('Toggle:', lang),
  },
};

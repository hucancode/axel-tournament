<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import LanguageSelector from './LanguageSelector.svelte';

  const { Story } = defineMeta({
    title: 'Components/LanguageSelector',
    component: LanguageSelector,
    args: {
      languages: ['rust', 'go', 'c'],
      selected: ['rust'],
      onToggle: (lang: string) => console.log('Toggle:', lang),
    },
  });
</script>

<Story name="Default" />

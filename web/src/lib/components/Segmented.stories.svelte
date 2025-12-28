<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Segmented from './Segmented.svelte';

  const { Story } = defineMeta({
    title: 'Data Entry/Segmented',
    component: Segmented,
  });
</script>

<Story
  name="Default"
  args={{
    options: [
      { value: 'list', label: 'List' },
      { value: 'grid', label: 'Grid' },
      { value: 'table', label: 'Table' }
    ],
    value: 'list'
  }}
/>

<Story
  name="Two Options"
  args={{
    options: [
      { value: 'light', label: 'Light' },
      { value: 'dark', label: 'Dark' }
    ],
    value: 'light'
  }}
/>

<Story
  name="Four Options"
  args={{
    options: [
      { value: 'all', label: 'All' },
      { value: 'active', label: 'Active' },
      { value: 'pending', label: 'Pending' },
      { value: 'completed', label: 'Completed' }
    ],
    value: 'all'
  }}
/>

<Story
  name="Disabled"
  args={{
    options: [
      { value: 'option1', label: 'Option 1' },
      { value: 'option2', label: 'Option 2' },
      { value: 'option3', label: 'Option 3' }
    ],
    value: 'option1',
    disabled: true
  }}
/>

<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Progress from './Progress.svelte';

  const { Story } = defineMeta({
    title: 'Components/Progress',
    component: Progress,
  });
</script>

<Story name="Default" args={{ value: 50 }}/>

<Story name="Primary" args={{ value: 75, variant: 'primary' }} />

<Story name="Secondary" args={{ value: 60, variant: 'secondary' }} />

<Story name="Success" args={{ value: 100, variant: 'success' }} />

<Story name="Error" args={{ value: 30, variant: 'error' }} />

<Story name="Accent" args={{ value: 45, variant: 'accent' }} />

<Story name="Almost Empty" args={{ value: 5, variant: 'primary' }} />

<Story name="Almost Full" args={{ value: 95, variant: 'success' }} />

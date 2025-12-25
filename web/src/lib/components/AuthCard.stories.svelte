<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import AuthCard from './AuthCard.svelte';

  const { Story } = defineMeta({
    title: 'Components/AuthCard',
    component: AuthCard,
    args: {
      title: 'Sign In',
      showEmailPassword: true,
      submitText: 'Sign In',
      onsubmit: () => console.log('Form submitted'),
    },
  });
</script>

<Story name="Default">
  {#snippet template(args)}
    <AuthCard {...args}>
      <p>Additional form content can go here</p>
    </AuthCard>
  {/snippet}
</Story>

<Story name="WithError" args={{ error: 'Invalid credentials' }} />
<Story name="Loading" args={{ loading: true }} />

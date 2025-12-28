<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Steps from './Steps.svelte';

  const { Story } = defineMeta({
    title: 'Navigation/Steps',
    component: Steps,
  });
</script>

<Story
  name="Default"
  args={{
    steps: [
      { label: 'Account Info', description: 'Enter your details' },
      { label: 'Verification', description: 'Verify your email' },
      { label: 'Complete', description: 'All done!' }
    ],
    current: 0
  }}
/>

<Story
  name="Step Two"
  args={{
    steps: [
      { label: 'Account Info', description: 'Enter your details' },
      { label: 'Verification', description: 'Verify your email' },
      { label: 'Complete', description: 'All done!' }
    ],
    current: 1
  }}
/>

<Story
  name="Completed"
  args={{
    steps: [
      { label: 'Account Info', description: 'Enter your details' },
      { label: 'Verification', description: 'Verify your email' },
      { label: 'Complete', description: 'All done!' }
    ],
    current: 2
  }}
/>

<Story
  name="Simple Labels"
  args={{
    steps: [
      { label: 'Choose Plan' },
      { label: 'Payment' },
      { label: 'Confirm' },
      { label: 'Success' }
    ],
    current: 1
  }}
/>

<Story
  name="Many Steps"
  args={{
    steps: [
      { label: 'Start', description: 'Begin process' },
      { label: 'Details', description: 'Enter information' },
      { label: 'Review', description: 'Check your data' },
      { label: 'Approve', description: 'Confirm submission' },
      { label: 'Submit', description: 'Send to server' },
      { label: 'Complete', description: 'All done!' }
    ],
    current: 2
  }}
/>

<Story
  name="Clickable"
  args={{
    steps: [
      { label: 'Personal Info', description: 'Name and email' },
      { label: 'Address', description: 'Shipping details' },
      { label: 'Payment', description: 'Card information' },
      { label: 'Review', description: 'Confirm order' }
    ],
    current: 1,
    clickable: true
  }}
/>

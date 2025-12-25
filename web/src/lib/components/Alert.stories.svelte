<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Alert from './Alert.svelte';

  const { Story } = defineMeta({
    title: 'Components/Alert',
    component: Alert,
  });
</script>

<Story name="Default" args={{ message: 'Operation failed successfully' }}/>

<Story name="Success" args={{ message: 'Operation completed successfully', type: 'success' }} />

<Story name="Warning" args={{ message: 'Please check your input', type: 'warning' }} />

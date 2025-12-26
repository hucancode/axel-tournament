<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import FormInputDemo from './FormInputDemo.svelte';

  const { Story } = defineMeta({
    title: 'Styles/Form Inputs',
    component: FormInputDemo,
  });
</script>

<Story name="Text Input" args={{ type: 'text', label: 'Username', placeholder: 'Enter your username' }} />

<Story name="Email Input" args={{ type: 'email', label: 'Email Address', placeholder: 'your@email.com' }} />

<Story name="Select" args={{ type: 'select', label: 'Tournament Type', options: ['Single Elimination', 'Double Elimination', 'Round Robin'] }} />

<Story name="Textarea" args={{ type: 'textarea', label: 'Description', placeholder: 'Enter description...' }} />

<Story name="With Error" args={{ type: 'text', label: 'Username', value: 'Invalid input', error: 'This field is required' }} />

<Story name="All Form Elements">
  <div class="max-w-lg">
    <FormInputDemo type="email" label="Email Address" placeholder="your@email.com" />
    <FormInputDemo type="select" label="Tournament Type" options={['Single Elimination', 'Double Elimination', 'Round Robin']} />
    <FormInputDemo type="textarea" label="Game Rules" placeholder="Describe the rules..." />
    <FormInputDemo type="text" label="Error Example" value="Invalid input" error="This field is required" />
    <button class="btn btn-primary">Submit</button>
  </div>
</Story>

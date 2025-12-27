<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Textarea from './Textarea.svelte';

  const { Story } = defineMeta({
    title: 'Form/Textarea',
    component: Textarea,
  });
</script>

<Story name="Default" args={{ label: 'Description', placeholder: 'Enter description...' }} />

<Story name="With Value" args={{ label: 'Game Rules', value: 'Tournament will follow standard rules.\nAll players must arrive 15 minutes early.\nNo outside assistance allowed.' }} />

<Story name="With Error" args={{ label: 'Description', value: '', error: 'This field is required' }} />

<Story name="Custom Rows" args={{ label: 'Long Text', placeholder: 'Enter a lot of text...', rows: 10 }} />

<Story name="Disabled" args={{ label: 'Read Only', value: 'This field is disabled', disabled: true }} />

<Story name="Multiple Textareas">
  <div class="max-w-lg">
    <Textarea label="Tournament Description" placeholder="Describe your tournament..." />
    <Textarea label="Special Rules" placeholder="Any special rules or notes..." rows={3} />
    <Textarea label="Prize Information" placeholder="Describe prizes..." />
    <button class="btn btn-primary">Submit</button>
  </div>
</Story>

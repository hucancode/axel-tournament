<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import FilterSelect from './FilterSelect.svelte';

  const statusOptions = [
    { value: "all", label: "All Tournaments" },
    { value: "scheduled", label: "Scheduled" },
    { value: "registration", label: "Registration Open" },
    { value: "running", label: "Running" },
    { value: "completed", label: "Completed" },
  ];

  const { Story } = defineMeta({
    title: 'Components/FilterSelect',
    component: FilterSelect,
    args: {
      label: 'Filter by Status:',
      options: statusOptions,
      value: 'all',
      disabled: false,
    },
  });
</script>

<Story name="Default" />

<Story name="Disabled" args={{ value: 'running', disabled: true }} />

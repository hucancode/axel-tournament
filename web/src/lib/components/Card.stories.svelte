<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Card from './Card.svelte';
  import Button from './Button.svelte';

  const { Story } = defineMeta({
    title: 'Data Display/Card',
    component: Card,
  });
</script>

<Story name="Basic Card" args={{ title: 'Card Title', content: 'This is a basic card with neubrutalism styling featuring thick borders and hard shadows.' }} />

<Story name="Card with Content">
  <div class="max-w-md">
    <Card>
      <h3 class="mb-4 text-2xl">Tournament Details</h3>
      <div class="mb-3">
        <strong>Name:</strong> Spring Championship 2024
      </div>
      <div class="mb-3">
        <strong>Date:</strong> March 15, 2024
      </div>
      <div class="mb-4">
        <strong>Players:</strong> 16/32
      </div>
      <Button variant="primary" label="Register Now" />
    </Card>
  </div>
</Story>

<Story name="Multiple Cards">
  <div class="grid grid-2">
    <Card title="Card 1" content="First card in a grid layout" />
    <Card title="Card 2" content="Second card in a grid layout" />
    <Card title="Card 3" content="Third card in a grid layout" />
  </div>
</Story>

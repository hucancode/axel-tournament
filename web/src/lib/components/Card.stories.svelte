<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import CardDemo from './CardDemo.svelte';
  import ButtonDemo from './ButtonDemo.svelte';

  const { Story } = defineMeta({
    title: 'Styles/Cards',
    component: CardDemo,
  });
</script>

<Story name="Basic Card" args={{ title: 'Card Title', content: 'This is a basic card with neubrutalism styling featuring thick borders and hard shadows.' }} />

<Story name="Card with Content">
  <div class="max-w-md">
    <CardDemo>
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
      <ButtonDemo variant="primary" label="Register Now" />
    </CardDemo>
  </div>
</Story>

<Story name="Multiple Cards">
  <div class="grid grid-2">
    <CardDemo title="Card 1" content="First card in a grid layout" />
    <CardDemo title="Card 2" content="Second card in a grid layout" />
    <CardDemo title="Card 3" content="Third card in a grid layout" />
  </div>
</Story>

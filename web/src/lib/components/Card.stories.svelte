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
  <div style="max-width: 400px;">
    <CardDemo>
      <h3 style="margin-bottom: 1rem; font-size: 1.5rem;">Tournament Details</h3>
      <div style="margin-bottom: 0.75rem;">
        <strong>Name:</strong> Spring Championship 2024
      </div>
      <div style="margin-bottom: 0.75rem;">
        <strong>Date:</strong> March 15, 2024
      </div>
      <div style="margin-bottom: 1rem;">
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

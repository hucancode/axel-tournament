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
  <section class="card-container">
    <Card>
      <h3>Tournament Details</h3>
      <div class="detail">
        <strong>Name:</strong> Spring Championship 2024
      </div>
      <div class="detail">
        <strong>Date:</strong> March 15, 2024
      </div>
      <div class="detail">
        <strong>Players:</strong> 16/32
      </div>
      <Button variant="primary" label="Register Now" />
    </Card>
  </section>
</Story>

<Story name="Multiple Cards">
  <section class="cards-grid">
    <Card title="Card 1" content="First card in a grid layout" />
    <Card title="Card 2" content="Second card in a grid layout" />
    <Card title="Card 3" content="Third card in a grid layout" />
  </section>
</Story>

<style>
  .card-container {
    max-width: 28rem;
  }

  h3 {
    margin-bottom: 1rem;
    font-size: 1.5rem;
    line-height: 2rem;
  }

  .detail {
    margin-bottom: 0.75rem;
  }

  .detail:last-of-type {
    margin-bottom: 1rem;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1rem;
  }
</style>

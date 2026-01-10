<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Card from './Card.svelte';

  const { Story } = defineMeta({
    title: 'Data Display/Card',
    component: Card,
  });
</script>

<Story name="Basic Card">
  {#snippet template()}
    <Card>
      <h3>Card Title</h3>
      <p>This is a basic card with neubrutalism styling featuring thick borders and hard shadows.</p>
    </Card>
  {/snippet}
</Story>

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
      <button data-variant="primary">Register Now</button>
    </Card>
  </section>
</Story>

<Story name="Multiple Cards">
  <section class="cards-grid">
    <Card>
      <h3>Card 1</h3>
      <p>First card in a grid layout</p>
    </Card>
    <Card>
      <h3>Card 2</h3>
      <p>Second card in a grid layout</p>
    </Card>
    <Card>
      <h3>Card 3</h3>
      <p>Third card in a grid layout</p>
    </Card>
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

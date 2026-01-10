<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Pagination from './Pagination.svelte';

  const { Story } = defineMeta({
    title: 'Navigation/Pagination',
    component: Pagination,
  });
</script>

<script lang="ts">
  let currentPage = $state(1);

  function handlePageChange(page: number) {
    currentPage = page;
    console.log('Page changed to:', page);
  }
</script>

<Story name="Default" args={{ currentPage: 1, totalPages: 10 }} />

<Story name="Middle Page" args={{ currentPage: 5, totalPages: 10 }} />

<Story name="Last Page" args={{ currentPage: 10, totalPages: 10 }} />

<Story name="Few Pages" args={{ currentPage: 2, totalPages: 5 }} />

<Story name="Many Pages" args={{ currentPage: 15, totalPages: 50 }} />

<Story name="Interactive">
  <section class="interactive-demo">
    <div class="status">
      <p>Current Page: <strong>{currentPage}</strong></p>
    </div>
    <Pagination
      currentPage={currentPage}
      totalPages={20}
      onPageChange={handlePageChange}
    />
  </section>
</Story>

<style>
  .interactive-demo {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .status {
    text-align: center;
  }

  .status p {
    margin-bottom: 0.5rem;
  }
</style>

<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Pagination from './Pagination.svelte';

  const { Story } = defineMeta({
    title: 'Components/Pagination',
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
  <div class="flex flex-col gap-4">
    <div class="text-center">
      <p class="mb-2">Current Page: <strong>{currentPage}</strong></p>
    </div>
    <Pagination
      currentPage={currentPage}
      totalPages={20}
      onPageChange={handlePageChange}
    />
  </div>
</Story>

<Story name="All Variants">
  <div class="flex flex-col gap-4">
    <div>
      <p class="mb-2 font-semibold">First Page (Prev disabled)</p>
      <Pagination currentPage={1} totalPages={10} />
    </div>

    <div>
      <p class="mb-2 font-semibold">Middle Page</p>
      <Pagination currentPage={5} totalPages={10} />
    </div>

    <div>
      <p class="mb-2 font-semibold">Last Page (Next disabled)</p>
      <Pagination currentPage={10} totalPages={10} />
    </div>

    <div>
      <p class="mb-2 font-semibold">Few Pages (No ellipsis)</p>
      <Pagination currentPage={3} totalPages={6} />
    </div>

    <div>
      <p class="mb-2 font-semibold">Many Pages (With ellipsis)</p>
      <Pagination currentPage={25} totalPages={100} />
    </div>
  </div>
</Story>

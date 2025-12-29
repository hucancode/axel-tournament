<script lang="ts">
  interface Props {
    currentPage?: number;
    totalPages?: number;
    onPageChange?: (page: number) => void;
  }

  let { currentPage = 1, totalPages = 10, onPageChange = () => {} }: Props = $props();

  function getPageNumbers() {
    const pages: (number | string)[] = [];
    const showEllipsis = totalPages > 7;

    if (!showEllipsis) {
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i);
      }
      return pages;
    }

    pages.push(1);

    if (currentPage > 3) {
      pages.push('...');
    }

    const start = Math.max(2, currentPage - 1);
    const end = Math.min(totalPages - 1, currentPage + 1);

    for (let i = start; i <= end; i++) {
      pages.push(i);
    }

    if (currentPage < totalPages - 2) {
      pages.push('...');
    }

    pages.push(totalPages);

    return pages;
  }

  let pages = $derived(getPageNumbers());
  let canGoPrevious = $derived(currentPage > 1);
  let canGoNext = $derived(currentPage < totalPages);

  function handlePageClick(page: number) {
    if (page >= 1 && page <= totalPages && page !== currentPage) {
      onPageChange(page);
    }
  }
</script>

<nav aria-label="Pagination" class="flex justify-center">
  <ul class="flex gap-1 list-none p-0 m-0 flex-wrap items-center">
    <li>
      <button
        data-nav
        disabled={!canGoPrevious}
        onclick={() => handlePageClick(currentPage - 1)}
        aria-label="Previous page"
        class="min-w-0 px-3 py-2 bg-blueprint-paper border border-border-strong font-medium text-sm cursor-pointer transition-all-muted hover:bg-gray-light hover:border-border-strong active:opacity-90 focus:outline-2 focus:outline-primary focus:outline-offset-2 focus:z-10 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-blueprint-paper disabled:hover:text-muted"
      >
        ← Prev
      </button>
    </li>

    {#each pages as page}
      <li>
        {#if page === '...'}
          <span class="flex items-center px-1 py-2 font-medium-muted">...</span>
        {:else}
          <button
            onclick={() => handlePageClick(page as number)}
            aria-label={`Page ${page}`}
            aria-current={page === currentPage ? 'page' : undefined}
            class="min-w-8 px-3 py-2 bg-blueprint-paper border border-border-strong font-medium text-sm cursor-pointer transition-all-muted hover:bg-gray-light hover:border-border-strong active:opacity-90 focus:outline-2 focus:outline-primary focus:outline-offset-2 focus:z-10 disabled:opacity-50 disabled:cursor-not-allowed {page === currentPage ? 'bg-primary border-primary text-white hover:bg-primary hover:border-primary hover:cursor-default' : ''}"
          >
            {page}
          </button>
        {/if}
      </li>
    {/each}

    <li>
      <button
        data-nav
        disabled={!canGoNext}
        onclick={() => handlePageClick(currentPage + 1)}
        aria-label="Next page"
        class="min-w-0 px-3 py-2 bg-blueprint-paper border border-border-strong font-medium text-sm cursor-pointer transition-all-muted hover:bg-gray-light hover:border-border-strong active:opacity-90 focus:outline-2 focus:outline-primary focus:outline-offset-2 focus:z-10 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-blueprint-paper disabled:hover:text-muted"
      >
        Next →
      </button>
    </li>
  </ul>
</nav>

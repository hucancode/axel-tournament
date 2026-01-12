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

<nav aria-label="Pagination">
  <ul>
    <li>
      <button
        data-nav
        disabled={!canGoPrevious}
        onclick={() => handlePageClick(currentPage - 1)}
        aria-label="Previous page"
      >
        ← Prev
      </button>
    </li>

    {#each pages as page}
      <li>
        {#if page === '...'}
          <span class="ellipsis">...</span>
        {:else}
          <button
            onclick={() => handlePageClick(page as number)}
            aria-label={`Page ${page}`}
            aria-current={page === currentPage ? 'page' : undefined}
            data-current={page === currentPage}
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
      >
        Next →
      </button>
    </li>
  </ul>
</nav>

<style>
  nav {
    display: flex;
    justify-content: center;
  }

  ul {
    display: flex;
    gap: 0.25rem;
    list-style: none;
    padding: 0;
    margin: 0;
    flex-wrap: wrap;
    align-items: center;
  }

  button {
    min-width: 2rem;
    padding: var(--spacing-2) var(--spacing-3);
    background: var(--color-bg-light);
    border: 1px solid var(--color-border);
    color: var(--color-fg);
    font-weight: 500;
    font-size: 0.875rem;
    cursor: pointer;
    transition: border-color var(--transition-fast);
  }

  button[data-nav] {
    min-width: 0;
  }

  button:hover:not(:disabled) {
    border-color: var(--color-fg-dim);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button[data-current="true"] {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: var(--color-bg-dark);
  }

  button[data-current="true"]:hover {
    cursor: default;
  }

  .ellipsis {
    display: flex;
    align-items: center;
    padding: var(--spacing-2) var(--spacing-1);
    font-weight: 500;
    color: var(--color-fg-dim);
  }
</style>

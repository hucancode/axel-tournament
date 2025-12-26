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

<nav class="pagination" aria-label="Pagination">
  <ul class="pagination-list">
    <li>
      <button
        class="pagination-btn pagination-prev"
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
          <span class="pagination-ellipsis">...</span>
        {:else}
          <button
            class="pagination-btn"
            class:active={page === currentPage}
            onclick={() => handlePageClick(page as number)}
            aria-label={`Page ${page}`}
            aria-current={page === currentPage ? 'page' : undefined}
          >
            {page}
          </button>
        {/if}
      </li>
    {/each}

    <li>
      <button
        class="pagination-btn pagination-next"
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
  .pagination {
    display: flex;
    justify-content: center;
    margin: 1.5rem 0;
  }

  .pagination-list {
    display: flex;
    gap: 0.5rem;
    list-style: none;
    padding: 0;
    margin: 0;
    flex-wrap: wrap;
    align-items: center;
  }

  .pagination-btn {
    min-width: 2.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--white);
    border: 3px solid var(--black);
    border-radius: 4px;
    font-weight: 700;
    font-size: 0.875rem;
    cursor: pointer;
    box-shadow: 3px 3px 0 0 var(--black);
    transition: none;
    color: var(--black);
  }

  .pagination-btn:hover:not(:disabled) {
    transform: translate(1px, 1px);
    box-shadow: 2px 2px 0 0 var(--black);
    background: var(--secondary);
  }

  .pagination-btn:active:not(:disabled) {
    transform: translate(3px, 3px);
    box-shadow: none;
  }

  .pagination-btn:focus {
    outline: 3px solid var(--primary);
    outline-offset: 2px;
  }

  .pagination-btn.active {
    background: var(--primary);
    color: var(--black);
  }

  .pagination-btn.active:hover {
    background: var(--primary);
    transform: none;
    box-shadow: 3px 3px 0 0 var(--black);
    cursor: default;
  }

  .pagination-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
    box-shadow: 2px 2px 0 0 var(--gray-medium);
    border-color: var(--gray-medium);
  }

  .pagination-ellipsis {
    display: flex;
    align-items: center;
    padding: 0.5rem;
    font-weight: 700;
    color: var(--gray-medium);
  }

  .pagination-prev,
  .pagination-next {
    min-width: auto;
  }
</style>

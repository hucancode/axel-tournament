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
        class="nav-btn"
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
          <span>...</span>
        {:else}
          <button
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
        class="nav-btn"
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
    margin: 1.5rem 0;
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
    padding: 0.5rem 0.75rem;
    background: var(--white);
    border: 1px solid var(--border-color-strong);
    border-radius: 0;
    font-weight: 500;
    font-size: 0.875rem;
    cursor: pointer;
    transition: border-color 0.15s ease, background-color 0.15s ease, color 0.15s ease;
    color: var(--text-muted);
  }

  button:hover:not(:disabled) {
    background: var(--gray-light);
    color: var(--text);
    border-color: var(--border-color-strong);
  }

  button:active:not(:disabled) {
    opacity: 0.9;
  }

  button:focus {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
    z-index: 1;
  }

  button.active {
    background: var(--primary);
    border-color: var(--primary);
    color: var(--white);
  }

  button.active:hover {
    background: var(--primary);
    border-color: var(--primary);
    cursor: default;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    color: var(--text-muted);
  }

  span {
    display: flex;
    align-items: center;
    padding: 0.5rem 0.25rem;
    font-weight: 500;
    color: var(--text-muted);
  }

  .nav-btn {
    min-width: auto;
  }
</style>

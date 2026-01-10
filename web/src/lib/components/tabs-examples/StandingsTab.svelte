<section>
  <h3>Current Standings</h3>
  <ol>
    <li data-rank="1">
      <span>Player A</span>
      <span>12 pts</span>
    </li>
    <li>
      <span>Player B</span>
      <span>10 pts</span>
    </li>
    <li>
      <span>Player C</span>
      <span>8 pts</span>
    </li>
  </ol>
</section>

<style>
  h3 {
    margin-bottom: 1rem;
    font-size: 1.25rem;
    font-weight: bold;
  }

  ol {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }

  li {
    display: flex;
    justify-content: space-between;
    padding: 0.75rem;
    background: var(--color-gray-light);
    border: 2px solid var(--black);
  }

  li[data-rank="1"] {
    background: var(--primary);
    font-weight: bold;
  }
</style>

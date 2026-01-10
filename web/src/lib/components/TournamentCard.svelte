<script lang="ts">
  import Badge from "$lib/components/Badge.svelte";
  import type { Tournament } from "$lib/types";
  let {
    tournament,
    participants = [],
    href,
    class: className = "",
  } = $props<{
    tournament: Tournament;
    participants?: { id: string }[];
    href?: string;
    class?: string;
  }>();
  let link = $derived(href ?? `/tournaments/tournament?id=${tournament.id}`);
</script>

<a href={link} class="bg-hatch {className}">
  <h3>{tournament.name}</h3>
  <p>{tournament.description}</p>
  <footer>
    <Badge status={tournament.status} label={tournament.status} />
    <span>
      {participants.length}/{tournament.max_players} players
    </span>
  </footer>
</a>

<style>
  a {
    display: block;
    border: 1px solid var(--color-gray-800);
    padding: 1.5rem;
    text-decoration: none;
    transition: border-color 0.15s ease;
  }

  a:hover {
    border-color: var(--border-strong);
  }

  h3 {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0 0 0.5rem 0;
  }

  p {
    font-size: 0.875rem;
    color: var(--color-gray-700);
    margin: 0 0 1rem 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    line-clamp: 2;
    overflow: hidden;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  footer span {
    font-size: 0.875rem;
    color: var(--color-gray-500);
  }
</style>

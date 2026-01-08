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

<a href={link} class={`block border border-gray-800 p-6 bg-hatch no-underline transition-all duration-150 hover:border-border-strong ${className}`}>
  <h3 class="text-lg font-semibold mb-2">{tournament.name}</h3>
  <p
    class="text-sm text-gray-700 mb-4 line-clamp-2"
  >
    {tournament.description}
  </p>
  <div class="flex items-center justify-between">
    <Badge status={tournament.status} label={tournament.status} />
    <span class="text-sm text-gray-500">
      {participants.length}/{tournament.max_players} players
    </span>
  </div>
</a>

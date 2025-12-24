<script lang="ts">
  import StatusBadge from "$lib/components/StatusBadge.svelte";
  import type { Tournament } from "$lib/types";

  let {
    tournament,
    href,
    class: className = "",
  } = $props<{
    tournament: Tournament;
    href?: string;
    class?: string;
  }>();

  let link = $derived(href ?? `/tournaments/${tournament.id}`);
  let cardClass = $derived(`card tournament-card ${className}`.trim());
</script>

<a href={link} class={cardClass}>
  <h3 class="text-lg font-semibold mb-2">{tournament.name}</h3>
  <p
    class="text-sm text-gray-700 mb-4"
    style="overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;"
  >
    {tournament.description}
  </p>
  <div class="flex items-center justify-between">
    <StatusBadge status={tournament.status} label={tournament.status} />
    <span class="text-sm text-gray-500">
      {tournament.current_players}/{tournament.max_players} players
    </span>
  </div>
</a>

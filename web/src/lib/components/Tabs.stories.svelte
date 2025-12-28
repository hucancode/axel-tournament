<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Tabs from './Tabs.svelte';

  const { Story } = defineMeta({
    title: 'Container/Tabs',
    component: Tabs,
  });
</script>

<script lang="ts">
  // Tab content components
  import MatchesTab from './tabs-examples/MatchesTab.svelte';
  import StandingsTab from './tabs-examples/StandingsTab.svelte';
  import PlayersTab from './tabs-examples/PlayersTab.svelte';
  import RulesTab from './tabs-examples/RulesTab.svelte';
</script>

<Story name="Tournament Tabs">
  {#snippet template()}
    <Tabs
      items={[
        { label: 'Matches', value: 'matches', component: MatchesTab },
        { label: 'Standings', value: 'standings', component: StandingsTab },
        { label: 'Players', value: 'players', component: PlayersTab },
        { label: 'Rules', value: 'rules', component: RulesTab },
      ]}
    />
  {/snippet}
</Story>

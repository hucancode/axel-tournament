<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import TournamentCard from './TournamentCard.svelte';
  import type { Tournament } from '$lib/types';

  const { Story } = defineMeta({
    title: 'Data Display/TournamentCard',
    component: TournamentCard,
  });

  const mockTournament: Tournament = {
    id: '1',
    game_id: 'game-1',
    name: 'Summer Championship 2024',
    description: 'Join us for the biggest tournament of the summer! Compete against the best players and win amazing prizes.',
    status: 'registration',
    min_players: 4,
    max_players: 16,
    start_time: '2024-07-15T10:00:00Z',
    end_time: '2024-07-15T18:00:00Z',
    match_generation_type: 'single_elimination',
    created_at: '2024-06-01T00:00:00Z',
    updated_at: '2024-06-01T00:00:00Z',
  };
</script>

<Story name="Default" args={{tournament:mockTournament,participants: [{ id: '1' }, { id: '2' }, { id: '3' }]}} />

<Story name="Full" args={{tournament: mockTournament, participants: Array.from({ length: 16 }, (_, i) => ({ id: `player-${i + 1}` }))}} />

<Story name="Running" args={{tournament: { ...mockTournament, status: 'running' }, participants: Array.from({ length: 12 }, (_, i) => ({ id: `player-${i + 1}` }))}} />

<Story name="Completed" args={{tournament: { ...mockTournament, status: 'completed' }, participants: Array.from({ length: 16 }, (_, i) => ({ id: `player-${i + 1}` }))}} />

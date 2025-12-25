import type { Meta, StoryObj } from "@storybook/svelte";
import { expect, within } from "storybook/test";
import TournamentCard from "$lib/components/TournamentCard.svelte";
import type { Tournament } from "$lib/types";

const baseTournament: Tournament = {
  id: "t-42",
  game_id: "g-1",
  name: "Axel Open",
  description: "Battle it out with adaptive bots in a weekend ladder.",
  status: "registration",
  min_players: 4,
  max_players: 32,
  start_time: "2025-06-01T12:00:00Z",
  end_time: "2025-06-01T18:00:00Z",
  match_generation_type: "round_robin",
  created_at: "2025-05-01T10:00:00Z",
  updated_at: "2025-05-10T10:00:00Z",
};

const baseParticipants = Array.from({ length: 12 }, (_, i) => ({ id: `p-${i}` }));

const meta = {
  title: "Components/TournamentCard",
  component: TournamentCard,
  args: {
    tournament: baseTournament,
    participants: baseParticipants,
  },
} satisfies Meta<typeof TournamentCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const RegistrationOpen: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await expect(canvas.getByText("Axel Open")).toBeInTheDocument();
    await expect(canvas.getByText("12/32 players")).toBeInTheDocument();
    await expect(canvas.getByText("registration")).toBeInTheDocument();
  },
};

export const NearCapacity: Story = {
  args: {
    tournament: {
      ...baseTournament,
      name: "Final Sprint",
      status: "running",
    },
    participants: Array.from({ length: 30 }, (_, i) => ({ id: `p-${i}` })),
  },
};

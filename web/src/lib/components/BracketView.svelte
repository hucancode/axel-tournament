<script lang="ts">
    import type { Match } from "$lib/models";

    interface Props {
        matches: Match[];
        usernameFor: (userId: string) => string;
    }

    let { matches, usernameFor }: Props = $props();

    /// Group bracket matches by (bracket_label, round). Pure layout —
    /// no api calls, no state. Bracket order is stable: winners first,
    /// losers below, grand_final + reset at bottom.
    const grouped = $derived.by(() => {
        type Group = { label: string; rounds: Match[][] };
        const order = ["winners", "losers", "grand_final", "grand_final_reset"];
        const map: Record<string, Match[][]> = {};
        for (const m of matches) {
            if (!m.bracket || m.round === null || m.round === undefined) continue;
            const r = map[m.bracket] ?? [];
            const idx = m.round;
            while (r.length <= idx) r.push([]);
            r[idx].push(m);
            map[m.bracket] = r;
        }
        // Sort matches within each round by bracket_position.
        for (const b of Object.keys(map)) {
            for (const round of map[b]) {
                round.sort(
                    (a, b) =>
                        (a.bracket_position ?? 0) - (b.bracket_position ?? 0),
                );
            }
        }
        const out: Group[] = [];
        for (const key of order) {
            if (map[key]) out.push({ label: key, rounds: map[key] });
        }
        return out;
    });

    function winnerIdx(m: Match): number | null {
        if (m.status !== "completed" && m.status !== "failed") return null;
        if (m.faulted_user_ids.length > 0) {
            // Faulted players lose. Surviving non-faulted player wins.
            const idx = m.participants.findIndex(
                (p) => p.user_id && !m.faulted_user_ids.includes(p.user_id),
            );
            return idx === -1 ? null : idx;
        }
        let best = 0;
        for (let i = 1; i < m.participants.length; i++) {
            if (
                (m.participants[i].score ?? 0) >
                (m.participants[best].score ?? 0)
            )
                best = i;
        }
        return best;
    }

    function bracketLabel(key: string): string {
        switch (key) {
            case "winners":
                return "Winners";
            case "losers":
                return "Losers";
            case "grand_final":
                return "Grand Final";
            case "grand_final_reset":
                return "Grand Final Reset";
            default:
                return key;
        }
    }
</script>

{#if grouped.length === 0}
    <p class="empty">No bracket matches yet.</p>
{:else}
    <div class="bracket-root">
        {#each grouped as g}
            <section class="bracket-group">
                <h3 class="bracket-label">{bracketLabel(g.label)}</h3>
                <div class="rounds">
                    {#each g.rounds as round, ri}
                        <div class="round-col">
                            <h4 class="round-label">R{ri}</h4>
                            {#each round as m}
                                {@const w = winnerIdx(m)}
                                <div class="match-card status-{m.status}">
                                    {#each m.participants as p, idx}
                                        <div
                                            class="row"
                                            class:winner={w === idx}
                                            class:faulted={p.user_id &&
                                                m.faulted_user_ids.includes(
                                                    p.user_id,
                                                )}
                                        >
                                            <span class="name"
                                                >{usernameFor(
                                                    p.user_id ?? "",
                                                )}</span
                                            >
                                            <span class="score">
                                                {p.score === null ||
                                                p.score === undefined
                                                    ? "-"
                                                    : p.score.toFixed(1)}
                                            </span>
                                        </div>
                                    {/each}
                                    {#if m.participants.length === 1}
                                        <div class="row bye">
                                            <span class="name">BYE</span>
                                        </div>
                                    {/if}
                                    {#if m.error_message}
                                        <div class="fault-line">
                                            ⚠ {m.error_message}
                                        </div>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    {/each}
                </div>
            </section>
        {/each}
    </div>
{/if}

<style>
    .empty {
        color: var(--color-fg-muted);
        text-align: center;
        padding: var(--spacing-4);
    }
    .bracket-root {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-6);
    }
    .bracket-label {
        font-size: 0.875rem;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: var(--color-fg-muted);
        margin-bottom: var(--spacing-2);
    }
    .rounds {
        display: flex;
        gap: var(--spacing-4);
        overflow-x: auto;
        padding-bottom: var(--spacing-2);
    }
    .round-col {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-3);
        min-width: 11rem;
    }
    .round-label {
        font-size: 0.75rem;
        color: var(--color-fg-muted);
        margin: 0;
    }
    .match-card {
        background-color: var(--color-bg-light);
        border-left: 3px solid var(--color-border);
        padding: var(--spacing-2);
        font-size: 0.875rem;
    }
    .status-completed {
        border-left-color: var(--color-success);
    }
    .status-failed {
        border-left-color: var(--color-error);
    }
    .status-running {
        border-left-color: var(--color-warning);
    }
    .row {
        display: flex;
        justify-content: space-between;
        padding: 2px 0;
    }
    .row.winner {
        font-weight: 700;
        color: var(--color-fg);
    }
    .row.faulted {
        text-decoration: line-through;
        color: var(--color-error);
    }
    .row.bye .name {
        font-style: italic;
        color: var(--color-fg-muted);
    }
    .fault-line {
        font-size: 0.75rem;
        color: var(--color-error);
        margin-top: 2px;
    }
    .name {
        flex: 1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .score {
        font-variant-numeric: tabular-nums;
        margin-left: var(--spacing-2);
    }
</style>

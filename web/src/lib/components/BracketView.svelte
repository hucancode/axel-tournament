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

    /// Single-participant non-pending = structural BYE (round 0, odd
    /// player count). Single-participant pending = waiting on feeder,
    /// render as TBD.
    function isStructuralBye(m: Match): boolean {
        return m.participants.length === 1 && m.status !== "pending";
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
                        <div class="round-wrap">
                            <h4 class="round-label">R{ri}</h4>
                            <div
                                class="round-col"
                                data-ri={ri}
                                data-has-next={ri < g.rounds.length - 1 ||
                                    undefined}
                            >
                                {#each round as m}
                                    {@const w = winnerIdx(m)}
                                    {@const bye = isStructuralBye(m)}
                                    {@const slots = bye
                                        ? m.participants
                                        : m.participants.length < 2
                                          ? [
                                                ...m.participants,
                                                ...Array(
                                                    2 - m.participants.length,
                                                ).fill(null),
                                            ]
                                          : m.participants}
                                    <div class="match-card status-{m.status}">
                                        {#each slots as p, idx}
                                            {#if p === null || !p.user_id}
                                                <div class="row tbd">
                                                    <span class="name">TBD</span>
                                                    <span class="score">-</span>
                                                </div>
                                            {:else}
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
                                            {/if}
                                        {/each}
                                        {#if bye}
                                            <div class="row bye">
                                                <span class="name">BYE</span>
                                            </div>
                                        {/if}
                                        {#if m.error_message}
                                            <div class="fault-line">
                                                <svg class="icon"
                                                    ><use
                                                        href="/icons.svg#i-warning"
                                                    /></svg>
                                                {m.error_message}
                                            </div>
                                        {/if}
                                    </div>
                                {/each}
                            </div>
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
        font-size: var(--font-size-sm);
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: var(--color-fg-muted);
        margin-bottom: var(--spacing-2);
    }
    .rounds {
        /* Inter-round horizontal spacing — fits one outgoing stub + a
           short pre-card stub (1.5rem each = 3rem total). */
        display: flex;
        gap: 3rem;
        overflow-x: auto;
        padding-bottom: var(--spacing-2);
    }
    .round-wrap {
        display: flex;
        flex-direction: column;
        min-width: 11rem;
    }
    .round-label {
        font-size: var(--font-size-xs);
        color: var(--color-fg-muted);
        margin: 0 0 var(--spacing-2) 0;
    }
    .round-col {
        /* Card height + base gap drive the vertical bracket geometry.
           Each round R doubles the spacing of round R-1 so card
           midpoints in round R+1 align with the midpoint of each pair
           below. Padding-top centres round R against round 0.
              pad_R  = (2^R - 1) * (H + G) / 2
              gap_R  = (2^R - 1) * H + 2^R * G
        */
        --H: 3.6rem;
        --G: var(--spacing-3);
        --pad-this: 0px;
        --gap-this: var(--G);
        display: flex;
        flex-direction: column;
        padding-top: var(--pad-this);
        gap: var(--gap-this);
    }
    .round-col[data-ri="1"] {
        --pad-this: calc((var(--H) + var(--G)) / 2);
        --gap-this: calc(var(--H) + 2 * var(--G));
    }
    .round-col[data-ri="2"] {
        --pad-this: calc(3 * (var(--H) + var(--G)) / 2);
        --gap-this: calc(3 * var(--H) + 4 * var(--G));
    }
    .round-col[data-ri="3"] {
        --pad-this: calc(7 * (var(--H) + var(--G)) / 2);
        --gap-this: calc(7 * var(--H) + 8 * var(--G));
    }
    .round-col[data-ri="4"] {
        --pad-this: calc(15 * (var(--H) + var(--G)) / 2);
        --gap-this: calc(15 * var(--H) + 16 * var(--G));
    }
    .round-col[data-ri="5"] {
        --pad-this: calc(31 * (var(--H) + var(--G)) / 2);
        --gap-this: calc(31 * var(--H) + 32 * var(--G));
    }
    .match-card {
        position: relative;
        background-color: var(--color-bg-light);
        border-left: 3px solid var(--color-border);
        padding: var(--spacing-2);
        font-size: var(--font-size-sm);
        height: var(--H);
        box-sizing: border-box;
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
    .row.tbd .name {
        font-style: italic;
        color: var(--color-fg-dim);
    }
    .fault-line {
        font-size: var(--font-size-xs);
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

    /* --- Connector lines ---
       Outgoing horizontal stub from every card in non-final rounds.
       Vertical fork joins each pair (odd→down, even→up); the even card
       also paints the horizontal mid-extension that lands on the
       next-round card's left edge. */
    .round-col[data-has-next] .match-card::after {
        content: "";
        position: absolute;
        left: 100%;
        top: 50%;
        width: 1.5rem;
        height: 1px;
        background: var(--color-border);
        pointer-events: none;
    }
    /* Odd card (1st of pair): vertical line from card mid down to pair mid. */
    .round-col[data-has-next]
        .match-card:nth-of-type(odd):not(:last-of-type)::before {
        content: "";
        position: absolute;
        left: calc(100% + 1.5rem);
        top: 50%;
        width: 1px;
        height: calc((var(--H) + var(--gap-this)) / 2);
        background: var(--color-border);
        pointer-events: none;
    }
    /* Even card (2nd of pair): vertical line from pair mid down to card mid
       + horizontal extension from pair mid into next-round card. */
    .round-col[data-has-next] .match-card:nth-of-type(even)::before {
        content: "";
        position: absolute;
        left: calc(100% + 1.5rem);
        bottom: 50%;
        width: 1.5rem;
        height: calc((var(--H) + var(--gap-this)) / 2);
        border-left: 1px solid var(--color-border);
        border-top: 1px solid var(--color-border);
        pointer-events: none;
    }
</style>

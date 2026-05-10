<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";
    import BracketView from "./BracketView.svelte";
    import type { Match } from "$lib/models";

    const { Story } = defineMeta({
        title: "Data Display/BracketView",
        component: BracketView,
        parameters: {
            layout: "fullscreen",
        },
    });

    const names: Record<string, string> = {
        u1: "alice",
        u2: "bob",
        u3: "carol",
        u4: "dave",
        u5: "eve",
        u6: "frank",
        u7: "grace",
        u8: "henry",
    };
    const usernameFor = (id: string) => names[id] ?? id;

    const now = new Date().toISOString();

    function match(
        bracket: string,
        round: number,
        position: number,
        a: string | null,
        b: string | null,
        scores: [number | null, number | null],
        status: Match["status"],
        opts: Partial<Match> = {},
    ): Match {
        const participants = [];
        if (a !== null) participants.push({ user_id: a, score: scores[0] ?? undefined });
        if (b !== null) participants.push({ user_id: b, score: scores[1] ?? undefined });
        return {
            id: `${bracket}-${round}-${position}`,
            game_id: "g1",
            status,
            participants,
            faulted_user_ids: [],
            round,
            bracket,
            bracket_position: position,
            created_at: now,
            updated_at: now,
            ...opts,
        };
    }

    const singleElim: Match[] = [
        match("winners", 0, 0, "u1", "u8", [3, 1], "completed"),
        match("winners", 0, 1, "u2", "u7", [2, 0], "completed"),
        match("winners", 0, 2, "u3", "u6", [1, 1], "failed", {
            error_message: "carol disconnected",
            faulted_user_ids: ["u3"],
        }),
        match("winners", 0, 3, "u4", "u5", [2, 3], "completed"),
        match("winners", 1, 0, "u1", "u2", [3, 2], "completed"),
        match("winners", 1, 1, "u6", "u5", [0, 0], "running"),
        match("winners", 2, 0, "u1", "u5", [null, null], "pending"),
    ];

    const doubleElim: Match[] = [
        ...singleElim,
        match("losers", 0, 0, "u8", "u7", [1, 2], "completed"),
        match("losers", 0, 1, "u4", "u3", [3, 0], "completed"),
        match("losers", 1, 0, "u7", "u4", [null, null], "pending"),
        match("grand_final", 0, 0, "u1", "u4", [null, null], "pending"),
    ];

    /// In real bracket generation, future-round matches are only
    /// materialised when their feeders complete — but the API may also
    /// pre-create stubs with one or zero known participants. Those stubs
    /// should render as TBD slots, not as byes.
    const earlyState: Match[] = [
        match("winners", 0, 0, "u1", "u8", [3, 1], "completed"),
        match("winners", 0, 1, "u2", "u7", [null, null], "running"),
        match("winners", 0, 2, "u3", "u6", [null, null], "pending"),
        match("winners", 0, 3, "u4", "u5", [null, null], "pending"),
        // R1: only one feeder known so far.
        match("winners", 1, 0, "u1", null, [null, null], "pending"),
        match("winners", 1, 1, null, null, [null, null], "pending"),
        // Final: nothing known yet.
        match("winners", 2, 0, null, null, [null, null], "pending"),
    ];

    const byeBracket: Match[] = [
        match("winners", 0, 0, "u1", null, [3, null], "completed"),
        match("winners", 0, 1, "u2", "u3", [2, 1], "completed"),
        match("winners", 1, 0, "u1", "u2", [null, null], "pending"),
    ];
</script>

<Story name="All Variants">
    {#snippet template()}
        <section class="showcase">
            <h2>BracketView</h2>
            <p class="lede">
                Renders single- and double-elimination brackets. Pending matches with
                unknown players show <code>TBD</code>; structural byes show <code>BYE</code>.
                Connector lines join each pair to its next-round match.
            </p>

            <h3>Single elimination — fully populated</h3>
            <BracketView matches={singleElim} {usernameFor} />

            <h3>Double elimination</h3>
            <BracketView matches={doubleElim} {usernameFor} />

            <h3>Early state — most slots TBD</h3>
            <BracketView matches={earlyState} {usernameFor} />

            <h3>Bye in round 0</h3>
            <BracketView matches={byeBracket} {usernameFor} />

            <h3>Empty</h3>
            <BracketView matches={[]} {usernameFor} />
        </section>
    {/snippet}
</Story>

<style>
    .showcase {
        padding: var(--spacing-4);
    }
    h2 {
        margin: 0 0 var(--spacing-2) 0;
    }
    h3 {
        margin: var(--spacing-6) 0 var(--spacing-3) 0;
        font-size: var(--font-size-md);
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--color-fg-muted);
    }
    .lede {
        color: var(--color-fg-muted);
        margin-bottom: var(--spacing-4);
    }
    code {
        font-family: monospace;
        font-size: var(--font-size-xs);
        color: var(--color-fg-muted);
    }
</style>

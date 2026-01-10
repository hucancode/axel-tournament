<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";
    const { Story } = defineMeta({
        title: "Container/Tabs",
    });
</script>

<Story name="Native Radio Tabs">
    {#snippet template()}
        <div>
            <nav>
                <input
                    type="radio"
                    id="matches"
                    name="tournament-tab"
                    role="tab"
                    checked
                />
                <label for="matches">Matches</label>

                <input type="radio" id="standings" name="tournament-tab" role="tab" />
                <label for="standings">Standings</label>

                <input type="radio" id="players" name="tournament-tab" role="tab" />
                <label for="players">Players</label>

                <input type="radio" id="rules" name="tournament-tab" role="tab" />
                <label for="rules">Rules</label>
            </nav>

            <section role="tabpanel">
                <h3>Match Schedule</h3>
                <p>
                    Example matches content using native radio tabs with
                    segmented styling
                </p>
            </section>
            <section role="tabpanel">
                <h3>Current Standings</h3>
                <p>
                    Example standings content using native radio tabs with
                    segmented styling
                </p>
            </section>
            <section role="tabpanel">
                <h3>Registered Players</h3>
                <p>
                    Example players content using native radio tabs with
                    segmented styling
                </p>
            </section>
            <section role="tabpanel">
                <h3>Tournament Rules</h3>
                <p>
                    Example rules content using native radio tabs with segmented
                    styling
                </p>
            </section>
        </div>
    {/snippet}
</Story>

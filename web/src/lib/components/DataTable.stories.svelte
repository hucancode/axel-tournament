<script module lang="ts">
    import { defineMeta } from "@storybook/addon-svelte-csf";
    import DataTable from "./DataTable.svelte";

    const { Story } = defineMeta({
        title: "Data Display/DataTable",
        component: DataTable,
    });

    const players = [
        { rank: 1, name: "alice", wins: 12, losses: 2, score: 1842 },
        { rank: 2, name: "bob", wins: 9, losses: 5, score: 1721 },
        { rank: 3, name: "carol", wins: 8, losses: 6, score: 1665 },
        { rank: 4, name: "dave", wins: 4, losses: 10, score: 1402 },
        { rank: 5, name: "eve", wins: 2, losses: 12, score: 1280 },
    ];

    const cols = [
        { key: "rank", label: "Rank", align: "right" as const },
        { key: "name", label: "Player" },
        { key: "wins", label: "W", align: "right" as const },
        { key: "losses", label: "L", align: "right" as const },
        { key: "score", label: "Score", align: "right" as const },
    ];
</script>

<Story name="Default">
    {#snippet template()}
        <DataTable columns={cols}>
            {#each players as p}
                <tr>
                    <td style:text-align="right">{p.rank}</td>
                    <td>{p.name}</td>
                    <td style:text-align="right">{p.wins}</td>
                    <td style:text-align="right">{p.losses}</td>
                    <td style:text-align="right">{p.score}</td>
                </tr>
            {/each}
        </DataTable>
    {/snippet}
</Story>

<Story name="No Header">
    {#snippet template()}
        <DataTable>
            {#each players as p}
                <tr>
                    <td>{p.name}</td>
                    <td style:text-align="right">{p.score}</td>
                </tr>
            {/each}
        </DataTable>
    {/snippet}
</Story>

<Story name="Empty">
    {#snippet template()}
        <DataTable columns={cols}>
            <tr>
                <td colspan={cols.length} style:text-align="center">
                    No players yet.
                </td>
            </tr>
        </DataTable>
    {/snippet}
</Story>

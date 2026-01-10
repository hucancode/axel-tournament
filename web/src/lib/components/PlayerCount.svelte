<script lang="ts">
    interface Props {
        current: number;
        min: number;
        max: number;
    }

    let { current, min, max }: Props = $props();

    function getBadgeType() {
        if (current >= max) return "full";
        if (current >= min) return "active";
        return "pending";
    }

    function getBadgeText() {
        if (current >= max) return "Full";
        if (current >= min) return "Active";
        return `Need ${min - current} more`;
    }

    let badgeType = $derived(getBadgeType());
    let badgeText = $derived(getBadgeText());
</script>

<div>
    {current} / {max}
    <span data-badge={badgeType}>
        {badgeText}
    </span>
</div>

<style>
    div {
        font-size: 0.875rem;
    }

    span {
        display: inline-block;
        padding: 0.25rem 0.75rem;
        margin-left: 0.5rem;
        font-size: 0.75rem;
        font-weight: 600;
        border-radius: 0.25rem;
        border: 1px solid;
    }

    span[data-badge="full"] {
        background: rgb(239 68 68 / 0.1);
        color: var(--error);
        border-color: var(--error);
    }

    span[data-badge="active"] {
        background: rgb(34 197 94 / 0.1);
        color: var(--success);
        border-color: var(--success);
    }

    span[data-badge="pending"] {
        background: rgb(234 179 8 / 0.1);
        color: var(--amber-600);
        border-color: var(--warning);
    }
</style>

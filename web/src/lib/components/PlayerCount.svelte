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

<div class="text-sm">
    {current} / {max}
    <span class="inline-block px-3 py-1 ml-2 text-xs font-semibold rounded border {badgeType === 'full' ? 'bg-red-500/10 text-error border-error' : badgeType === 'active' ? 'bg-emerald-500/10 text-success border-success' : 'bg-yellow-500/10 text-amber-600 border-warning'}">
        {badgeText}
    </span>
</div>

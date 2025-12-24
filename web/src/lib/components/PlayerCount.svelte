<script lang="ts">
    interface Props {
        current: number;
        min: number;
        max: number;
    }
    
    let { current, min, max }: Props = $props();
    
    function getBadgeInfo() {
        if (current >= max) {
            return { text: "Full", class: "badge-failed" };
        } else if (current >= min) {
            return { text: "Active", class: "badge-accepted" };
        } else {
            return { text: `Need ${min - current} more`, class: "badge-pending" };
        }
    }
    
    let badgeInfo = $derived(getBadgeInfo());
</script>

<div class="text-sm">
    {current} / {max}
    <span class="badge {badgeInfo.class}" style="margin-left: 0.5rem;">
        {badgeInfo.text}
    </span>
</div>

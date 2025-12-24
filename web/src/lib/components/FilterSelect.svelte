<script lang="ts">
    interface Option {
        value: string;
        label: string;
    }
    
    interface Props {
        label: string;
        options: Option[];
        value: string;
        disabled?: boolean;
        onchange?: () => void;
    }
    
    let { label, options, value = $bindable(""), disabled = false, onchange }: Props = $props();
</script>

<div class="flex items-center gap-4">
    <label for="filter-select" class="font-semibold">{label}</label>
    <select
        id="filter-select"
        class="select"
        style="width: auto;"
        bind:value
        {onchange}
        {disabled}
    >
        {#each options as option}
            <option value={option.value}>{option.label}</option>
        {/each}
    </select>
</div>

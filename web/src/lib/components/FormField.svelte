<script lang="ts">
    interface Props {
        label: string;
        type: string;
        id: string;
        value: string;
        required?: boolean;
        disabled?: boolean;
        placeholder?: string;
        minlength?: number;
        maxlength?: number;
        helpText?: string;
    }
    
    let { 
        label, 
        type, 
        id, 
        value = $bindable(""), 
        required = false, 
        disabled = false, 
        placeholder = "", 
        minlength, 
        maxlength, 
        helpText 
    }: Props = $props();
</script>

<div class="form-group">
    <label for={id}>{label}</label>
    <input
        {type}
        {id}
        class="input"
        bind:value
        {required}
        {disabled}
        {placeholder}
        {minlength}
        {maxlength}
    />
    {#if helpText}
        <p class="text-sm text-gray-500" style="margin-top: 0.25rem;">
            {helpText}
        </p>
    {/if}
</div>

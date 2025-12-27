<script lang="ts">
    interface Props {
        variant?: "primary" | "secondary" | "danger" | "success";
        disabled?: boolean;
        label?: string;
        type?: "button" | "submit" | "reset";
        onclick?: (e: MouseEvent) => void;
    }

    let {
        variant = "primary",
        disabled = false,
        label = "Button",
        type = "button",
        onclick,
    }: Props = $props();
</script>

<button class={variant} {disabled} {type} {onclick}>
    {label}
</button>

<style>
    button {
        padding: 0.75rem 1.5rem;
        border-radius: 4px;
        font-weight: 700;
        border: 3px solid var(--black);
        cursor: pointer;
        text-decoration: none;
        display: inline-block;
        box-shadow: 4px 4px 0 0 var(--black);
        transition: none;
    }

    button:hover {
        transform: translate(2px, 2px);
        box-shadow: 2px 2px 0 0 var(--black);
    }

    button:active {
        transform: translate(4px, 4px);
        box-shadow: none;
    }

    button:focus {
        outline: 3px solid var(--primary);
        outline-offset: 2px;
    }

    button.primary {
        background-color: var(--primary);
        color: var(--black);
    }

    button.primary:hover {
        background-color: var(--primary);
    }

    button.primary:focus {
        outline-color: var(--primary);
    }

    button.secondary {
        background-color: var(--secondary);
        color: var(--black);
    }

    button.secondary:hover {
        background-color: var(--secondary);
    }

    button.danger {
        background-color: var(--error);
        color: var(--white);
    }

    button.danger:hover {
        background-color: var(--error);
    }

    button.success {
        background-color: var(--success);
        color: var(--black);
    }

    button.success:hover {
        background-color: var(--success);
    }

    button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        transform: none;
        box-shadow: 2px 2px 0 0 var(--gray-medium);
        border-color: var(--gray-medium);
    }
</style>

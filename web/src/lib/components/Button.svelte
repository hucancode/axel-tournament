<script lang="ts">
    interface Props {
        variant?: "primary" | "secondary" | "danger" | "success" | "ghost";
        disabled?: boolean;
        label?: string;
        type?: "button" | "submit" | "reset";
        value?: string;
        onclick?: (e: MouseEvent) => void;
    }

    let {
        variant = "primary",
        disabled = false,
        label = "Button",
        type = "button",
        value,
        onclick,
    }: Props = $props();
</script>

<button data-variant={variant} {disabled} {type} {value} {onclick}>
    {label}
</button>

<style>
    button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0.5rem 1rem;
        font-size: 0.875rem;
        font-weight: 500;
        border: 1px solid;
        transition: color var(--transition-fast), background-color var(--transition-fast), border-color var(--transition-fast);
        cursor: pointer;
        user-select: none;
    }

    button:focus-visible {
        outline: 2px solid var(--color-gray-600);
        outline-offset: 1px;
    }

    button:disabled {
        opacity: 0.5;
        pointer-events: none;
        cursor: not-allowed;
    }

    button:active:not(:disabled) {
        opacity: 0.9;
    }

    /* Primary variant */
    button[data-variant="primary"] {
        background-color: var(--color-gray-800);
        color: white;
        border-color: var(--color-gray-800);
    }

    button[data-variant="primary"]:hover:not(:disabled) {
        background-color: var(--color-gray-700);
        border-color: var(--color-gray-700);
    }

    @media (prefers-color-scheme: dark) {
        button[data-variant="primary"] {
            background-color: var(--color-gray-200);
            color: var(--color-gray-900);
            border-color: var(--color-gray-200);
        }

        button[data-variant="primary"]:hover:not(:disabled) {
            background-color: var(--color-gray-300);
            border-color: var(--color-gray-300);
        }
    }

    /* Secondary variant */
    button[data-variant="secondary"] {
        background-color: white;
        color: var(--color-gray-800);
        border-color: var(--color-gray-200);
    }

    button[data-variant="secondary"]:hover:not(:disabled) {
        background-color: var(--color-gray-50);
    }

    @media (prefers-color-scheme: dark) {
        button[data-variant="secondary"] {
            background-color: var(--color-gray-800);
            color: var(--color-gray-200);
            border-color: var(--color-gray-700);
        }

        button[data-variant="secondary"]:hover:not(:disabled) {
            background-color: var(--color-gray-700);
        }
    }

    /* Ghost variant */
    button[data-variant="ghost"] {
        background-color: transparent;
        color: var(--color-gray-600);
        border-color: transparent;
    }

    button[data-variant="ghost"]:hover:not(:disabled) {
        background-color: var(--color-gray-100);
        color: var(--color-gray-800);
    }

    @media (prefers-color-scheme: dark) {
        button[data-variant="ghost"] {
            color: var(--color-gray-400);
        }

        button[data-variant="ghost"]:hover:not(:disabled) {
            background-color: var(--color-gray-800);
            color: var(--color-gray-200);
        }
    }

    /* Danger variant */
    button[data-variant="danger"] {
        background-color: #991B1B;
        color: white;
        border-color: #991B1B;
    }

    button[data-variant="danger"]:hover:not(:disabled) {
        background-color: #7F1D1D;
        border-color: #7F1D1D;
    }

    /* Success variant */
    button[data-variant="success"] {
        background-color: #065F46;
        color: white;
        border-color: #065F46;
    }

    button[data-variant="success"]:hover:not(:disabled) {
        background-color: #064E3B;
        border-color: #064E3B;
    }
</style>

<script lang="ts">
    interface Props {
        value?: number;
        max?: number;
        variant?: "primary" | "secondary" | "success" | "error" | "accent";
        labelPosition?: "middle" | "top" | "right";
    }

    let {
        value,
        max = 100,
        variant = "primary",
        labelPosition,
    }: Props = $props();

    let indeterminate = $derived(value === undefined);
    let percentage = $derived(
        indeterminate
            ? 0
            : Math.min(100, Math.max(0, value ? (value / max) * 100 : 0)),
    );
    let isComplete = $derived(percentage >= 100);
</script>

<div data-position={labelPosition}>
    <progress
        {value}
        {max}
        data-variant={variant}
        data-indeterminate={indeterminate || undefined}
    ></progress>
    <span
        data-value="{Math.round(percentage)}%"
        data-complete={isComplete || undefined}
    ></span>
</div>

<style>
    div {
        width: 100%;
        position: relative;
    }

    div[data-position="top"] {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    div[data-position="top"] > span {
        order: -1;
        text-align: right;
    }

    div[data-position="right"] {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    div[data-position="right"] > span {
        min-width: 3rem;
        text-align: right;
    }

    div[data-position="middle"] > span {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        pointer-events: none;
        color: var(--white);
        mix-blend-mode: difference;
    }

    progress {
        flex: 1;
        width: 100%;
        height: 1.25rem;
        border: 1px solid var(--border-color-strong);
        appearance: none;
    }

    progress::-webkit-progress-bar {
        background-color: var(--gray-light);
    }

    progress::-webkit-progress-value,
    progress::-moz-progress-bar {
        background-size: 8px 8px;
        background-image: repeating-linear-gradient(
            315deg,
            rgb(255 255 255 / 0.15) 0,
            rgb(255 255 255 / 0.15) 1px,
            transparent 0,
            transparent 50%
        );
    }

    progress[data-variant="primary"]::-webkit-progress-value,
    progress[data-variant="primary"]::-moz-progress-bar {
        background-color: rgb(51 65 85);
    }

    progress[data-variant="secondary"]::-webkit-progress-value,
    progress[data-variant="secondary"]::-moz-progress-bar {
        background-color: rgb(100 116 139);
    }

    progress[data-variant="success"]::-webkit-progress-value,
    progress[data-variant="success"]::-moz-progress-bar {
        background-color: rgb(22 101 52);
    }

    progress[data-variant="error"]::-webkit-progress-value,
    progress[data-variant="error"]::-moz-progress-bar {
        background-color: rgb(153 27 27);
    }

    progress[data-variant="accent"]::-webkit-progress-value,
    progress[data-variant="accent"]::-moz-progress-bar {
        background-color: rgb(30 64 175);
    }

    /* Indeterminate */
    progress[data-indeterminate]::-webkit-progress-bar {
        background-size: 8px 8px;
        background-image: repeating-linear-gradient(
            315deg,
            rgb(255 255 255 / 0.15) 0,
            rgb(255 255 255 / 0.15) 1px,
            transparent 0,
            transparent 50%
        );
        animation: hatch-scroll 0.5s linear infinite;
    }

    progress[data-indeterminate][data-variant="primary"]::-webkit-progress-bar {
        background-color: rgb(51 65 85);
    }
    progress[data-indeterminate][data-variant="secondary"]::-webkit-progress-bar {
        background-color: rgb(100 116 139);
    }
    progress[data-indeterminate][data-variant="success"]::-webkit-progress-bar {
        background-color: rgb(22 101 52);
    }
    progress[data-indeterminate][data-variant="error"]::-webkit-progress-bar {
        background-color: rgb(153 27 27);
    }
    progress[data-indeterminate][data-variant="accent"]::-webkit-progress-bar {
        background-color: rgb(30 64 175);
    }

    progress[data-indeterminate]::-moz-progress-bar {
        background: transparent;
    }

    progress[data-indeterminate]:indeterminate {
        background-size: 8px 8px;
        background-image: repeating-linear-gradient(
            315deg,
            rgb(255 255 255 / 0.15) 0,
            rgb(255 255 255 / 0.15) 1px,
            transparent 0,
            transparent 50%
        );
        animation: hatch-scroll 0.5s linear infinite;
    }

    progress[data-indeterminate][data-variant="primary"]:indeterminate {
        background-color: rgb(51 65 85);
    }
    progress[data-indeterminate][data-variant="secondary"]:indeterminate {
        background-color: rgb(100 116 139);
    }
    progress[data-indeterminate][data-variant="success"]:indeterminate {
        background-color: rgb(22 101 52);
    }
    progress[data-indeterminate][data-variant="error"]:indeterminate {
        background-color: rgb(153 27 27);
    }
    progress[data-indeterminate][data-variant="accent"]:indeterminate {
        background-color: rgb(30 64 175);
    }

    @keyframes hatch-scroll {
        to {
            background-position: 8px 8px;
        }
    }

    /* Label - hidden by default, shown via data-position */
    span {
        display: none;
        height: 1.25rem;
        font-size: 0.875rem;
        font-weight: 600;
        color: var(--text);
        position: relative;
        align-items: center;
        justify-content: center;
    }

    div[data-position] > span {
        display: inline-flex;
    }

    /* Percentage text via ::before */
    span::before {
        content: attr(data-value);
    }

    /* Complete state - hide percentage, show checkmark */
    span[data-complete] {
        animation: pop-in 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    }

    span[data-complete]::before {
        content: "✓";
        color: rgb(22 101 52);
        font-size: 1.25em;
        animation: checkmark-draw 0.3s ease-out 0.1s both;
    }

    div[data-position="middle"] span[data-complete]::before {
        color: var(--white);
    }

    /* Bubble */
    span[data-complete]::after {
        content: "";
        position: absolute;
        width: 2rem;
        height: 2rem;
        border-radius: 50%;
        border: 2px solid rgb(22 101 52);
        animation: bubble-expand 0.6s ease-out forwards;
    }

    div[data-position="middle"] span[data-complete]::after {
        border-color: var(--white);
    }

    @keyframes pop-in {
        0% {
            transform: scale(0.5);
            opacity: 0;
        }
        50% {
            transform: scale(1.2);
        }
        100% {
            transform: scale(1);
            opacity: 1;
        }
    }

    @keyframes checkmark-draw {
        0% {
            opacity: 0;
            transform: scale(0) rotate(-45deg);
        }
        50% {
            opacity: 1;
            transform: scale(1.3) rotate(0deg);
        }
        100% {
            opacity: 1;
            transform: scale(1) rotate(0deg);
        }
    }

    @keyframes bubble-expand {
        0% {
            transform: scale(0.5);
            opacity: 0.8;
        }
        100% {
            transform: scale(2);
            opacity: 0;
        }
    }
</style>

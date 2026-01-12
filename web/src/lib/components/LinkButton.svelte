<script lang="ts">
	interface Props {
		variant?: "primary" | "secondary" | "danger" | "success" | "ghost";
		href: string;
		label?: string;
	}

	let {
		variant = "primary",
		href,
		label = "Button",
	}: Props = $props();
</script>

<a {href} data-variant={variant}>
	{label}
</a>

<style>
	a {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: var(--spacing-2) var(--spacing-4);
		font-size: 1rem;
		font-weight: 500;
		border: 1px solid var(--color-border);
		background: var(--color-bg-light);
		color: var(--color-fg);
		text-decoration: none;
		transition: border-color var(--transition-fast);
		cursor: pointer;
	}

	a:hover {
		border-color: var(--color-fg-dim);
	}

	a[data-variant="primary"] {
		background: var(--color-primary);
		color: var(--color-bg-dark);
		border-color: var(--color-primary);
	}

	a[data-variant="primary"]:hover {
		border-color: var(--color-secondary);
	}

	a[data-variant="secondary"] {
		background: var(--color-bg-popup);
		border-color: var(--color-border);
	}

	a[data-variant="ghost"] {
		background: transparent;
		color: var(--color-fg-muted);
		border-color: transparent;
	}

	a[data-variant="ghost"]:hover {
		background: var(--color-bg-light);
		border-color: var(--color-border-light);
	}

	a[data-variant="danger"] {
		background: var(--color-error);
		color: var(--color-fg);
		border-color: var(--color-error);
	}

	a[data-variant="success"] {
		background: var(--color-success);
		color: var(--color-bg-dark);
		border-color: var(--color-success);
	}
</style>

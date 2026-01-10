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
		padding: 0.5rem 1rem;
		font-size: 0.875rem;
		font-weight: 500;
		border: 1px solid;
		text-decoration: none;
		transition: color var(--transition-fast), background-color var(--transition-fast), border-color var(--transition-fast);
		cursor: pointer;
		user-select: none;
	}

	a:focus-visible {
		outline: 2px solid var(--color-gray-600);
		outline-offset: 1px;
	}

	a:active {
		opacity: 0.9;
	}

	/* Primary variant */
	a[data-variant="primary"] {
		background-color: var(--color-gray-800);
		color: white;
		border-color: var(--color-gray-800);
	}

	a[data-variant="primary"]:hover {
		background-color: var(--color-gray-700);
		border-color: var(--color-gray-700);
	}

	@media (prefers-color-scheme: dark) {
		a[data-variant="primary"] {
			background-color: var(--color-gray-200);
			color: var(--color-gray-900);
			border-color: var(--color-gray-200);
		}

		a[data-variant="primary"]:hover {
			background-color: var(--color-gray-300);
			border-color: var(--color-gray-300);
		}
	}

	/* Secondary variant */
	a[data-variant="secondary"] {
		background-color: white;
		color: var(--color-gray-800);
		border-color: var(--color-gray-200);
	}

	a[data-variant="secondary"]:hover {
		background-color: var(--color-gray-50);
	}

	@media (prefers-color-scheme: dark) {
		a[data-variant="secondary"] {
			background-color: var(--color-gray-800);
			color: var(--color-gray-200);
			border-color: var(--color-gray-700);
		}

		a[data-variant="secondary"]:hover {
			background-color: var(--color-gray-700);
		}
	}

	/* Ghost variant */
	a[data-variant="ghost"] {
		background-color: transparent;
		color: var(--color-gray-600);
		border-color: transparent;
	}

	a[data-variant="ghost"]:hover {
		background-color: var(--color-gray-100);
		color: var(--color-gray-800);
	}

	@media (prefers-color-scheme: dark) {
		a[data-variant="ghost"] {
			color: var(--color-gray-400);
		}

		a[data-variant="ghost"]:hover {
			background-color: var(--color-gray-800);
			color: var(--color-gray-200);
		}
	}

	/* Danger variant */
	a[data-variant="danger"] {
		background-color: #991B1B;
		color: white;
		border-color: #991B1B;
	}

	a[data-variant="danger"]:hover {
		background-color: #7F1D1D;
		border-color: #7F1D1D;
	}

	/* Success variant */
	a[data-variant="success"] {
		background-color: #065F46;
		color: white;
		border-color: #065F46;
	}

	a[data-variant="success"]:hover {
		background-color: #064E3B;
		border-color: #064E3B;
	}
</style>

<script lang="ts">
	interface Props {
		variant?: "primary" | "secondary" | "danger" | "success";
		href: string;
		label?: string;
	}

	let {
		variant = "primary",
		href,
		label = "Button",
	}: Props = $props();

</script>

<a {href} class={variant}>
	{label}
</a>

<style>
	a {
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

	a:hover {
		transform: translate(2px, 2px);
		box-shadow: 2px 2px 0 0 var(--black);
	}

	a:active {
		transform: translate(4px, 4px);
		box-shadow: none;
	}

	a:focus {
		outline: 3px solid var(--primary);
		outline-offset: 2px;
	}

	a.primary {
		background-color: var(--primary);
		color: var(--black);
	}

	a.primary:hover {
		background-color: var(--primary);
	}

	a.primary:focus {
		outline-color: var(--primary);
	}

	a.secondary {
		background-color: var(--secondary);
		color: var(--black);
	}

	a.secondary:hover {
		background-color: var(--secondary);
	}

	a.danger {
		background-color: var(--error);
		color: var(--white);
	}

	a.danger:hover {
		background-color: var(--error);
	}

	a.success {
		background-color: var(--success);
		color: var(--black);
	}

	a.success:hover {
		background-color: var(--success);
	}
</style>

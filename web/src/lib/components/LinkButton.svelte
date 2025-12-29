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

	const baseClasses = "inline-flex items-center justify-center px-4 py-2 text-sm font-medium border no-underline transition-colors focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-slate-600";

	const variantClasses = {
		primary: "bg-slate-800 text-white border-slate-800 hover:bg-slate-700 hover:border-slate-700 dark:bg-slate-200 dark:text-slate-900 dark:border-slate-200 dark:hover:bg-slate-300",
		secondary: "bg-white text-slate-800 border-slate-200 hover:bg-slate-50 dark:bg-slate-800 dark:text-slate-200 dark:border-slate-700 dark:hover:bg-slate-700",
		ghost: "bg-transparent text-slate-600 border-transparent hover:bg-slate-100 hover:text-slate-800 dark:text-slate-400 dark:hover:bg-slate-800 dark:hover:text-slate-200",
		danger: "bg-red-800 text-white border-red-800 hover:bg-red-900 hover:border-red-900",
		success: "bg-emerald-800 text-white border-emerald-800 hover:bg-emerald-900 hover:border-emerald-900",
	};
</script>

<a {href} class="{baseClasses} {variantClasses[variant]}">
	{label}
</a>

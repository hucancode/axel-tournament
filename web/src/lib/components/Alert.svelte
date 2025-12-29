<script lang="ts">
    interface Props {
        message: string;
        type?: "error" | "success" | "warning";
        onclose?: () => void;
    }

    let { message, type = "error", onclose }: Props = $props();

    const typeClasses = {
        error: "bg-error/10 text-error border-l-2 border-l-error",
        success: "bg-success/10 text-success border-l-2 border-l-success",
        warning: "bg-warning/10 text-amber-600 border-l-2 border-l-warning",
    };
</script>

<aside class="mb-4 border border-border-strong relative pr-10 pl-4 py-4 {typeClasses[type]}" role="alert">
    <p class="m-0 font-medium">{message}</p>
    <button hidden={!onclose} onclick={onclose} aria-label="Close alert" class="absolute top-1/2 right-2 -translate-y-1/2 bg-transparent border-0 text-xl font-bold cursor-pointer p-0 w-6 h-6 flex items-center justify-center leading-none opacity-60 transition-opacity hover:opacity-100">×</button>
</aside>

<script lang="ts">
    interface Props {
        message: string;
        type?: "error" | "success" | "warning";
    }
    
    let { message, type = "error" }: Props = $props();
    
    const bgColors = {
        error: "#fee2e2",
        success: "#dcfce7", 
        warning: "#fef3c7"
    };
    
    const textColors = {
        error: "text-red-600",
        success: "text-green-600",
        warning: "text-yellow-600"
    };
</script>

<div class="card" style="background: {bgColors[type]}; margin-bottom: 1rem;">
    <p class={textColors[type]}>{message}</p>
</div>

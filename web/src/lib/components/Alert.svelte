<script lang="ts">
    interface Props {
        message: string;
        type?: "error" | "success" | "warning";
    }

    let { message, type = "error" }: Props = $props();
</script>

<div class="card alert alert-{type}">
    <p>{message}</p>
</div>

<style>
    .alert {
        margin-bottom: 1rem;
        border-radius: 4px;
        border: 3px solid #000;
        box-shadow: 4px 4px 0 0 #000;
    }

    .alert-error {
        background: #FF4444;
        color: #fff;
    }

    .alert-success {
        background: #00FF88;
        color: #000;
    }

    .alert-warning {
        background: #FFFF00;
        color: #000;
    }

    .alert p {
        margin: 0;
        font-weight: 600;
    }
</style>

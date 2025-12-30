<script lang="ts">
    type Variant = "neutral" | "positive" | "negative";

    interface Props {
        value: number | string;
        label: string;
        variant?: Variant;
    }

    let { value, label, variant = "neutral" }: Props = $props();

    function getVariantClass(variant: Variant): string {
        switch (variant) {
            case "positive":
                return "text-green-600";
            case "negative":
                return "text-red-600";
            case "neutral":
            default:
                return "text-primary";
        }
    }
</script>

<div class="p-6 bg-hatch text-center">
    <div class="text-3xl font-bold {getVariantClass(variant)}">
        {value}
    </div>
    <div class="text-sm text-gray-600 mt-2">{label}</div>
</div>

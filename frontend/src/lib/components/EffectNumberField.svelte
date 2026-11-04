<script lang="ts">
    import NumberInput from '$lib/components/NumberInput.svelte';

    interface Props {
        label: string;
        value?: number;
        min?: number;
        max?: number;
        step?: number;
        onchange?: (value: number) => void;
    }

    let { label, value = $bindable(), min, max, step, onchange }: Props = $props();
</script>

<label class="grid gap-1.5 text-sm">
    <span class="text-foreground-muted">{label}</span>
    <NumberInput {min} {max} {step} bind:value {onchange} />
</label>

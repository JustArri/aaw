<script lang="ts">
    interface Props {
        id?: string;
        value?: number;
        min?: number;
        max?: number;
        step?: number;
        disabled?: boolean;
        required?: boolean;
        ariaLabel?: string;
        class?: string;
        onchange?: (value: number) => void;
    }

    let {
        id,
        value = $bindable(),
        min,
        max,
        step,
        disabled = false,
        required = false,
        ariaLabel,
        class: className = '',
        onchange
    }: Props = $props();
</script>

<input
    class={`border-border-strong bg-surface-inset text-foreground h-10 min-w-0 rounded-lg border px-3 font-mono text-sm tabular-nums disabled:cursor-not-allowed disabled:opacity-50 ${className}`}
    {id}
    type="number"
    {min}
    {max}
    {step}
    {disabled}
    {required}
    aria-label={ariaLabel}
    bind:value
    onchange={(event) => onchange?.(event.currentTarget.valueAsNumber)} />

<script lang="ts">
    export type SegmentOption = {
        value: string;
        label: string;
        disabled?: boolean;
    };

    let {
        name,
        options,
        value = $bindable(),
        ariaLabel,
        compact = false,
        onchange
    }: {
        name: string;
        options: SegmentOption[];
        value?: string;
        ariaLabel: string;
        compact?: boolean;
        onchange?: (value: string) => void;
    } = $props();
</script>

<div
    class="border-border-strong bg-surface-inset grid overflow-hidden rounded-lg border"
    style:grid-template-columns={`repeat(${options.length}, minmax(0, 1fr))`}
    role="radiogroup"
    aria-label={ariaLabel}>
    {#each options as option, index (option.value)}
        <label
            class={`has-focus-visible:outline-accent relative grid cursor-pointer place-items-center px-1 text-center font-semibold transition-colors has-focus-visible:z-1 has-focus-visible:outline-2 has-focus-visible:-outline-offset-2 ${compact ? 'min-h-9 text-sm' : 'min-h-10 text-xs'} ${index > 0 ? 'border-border-strong border-l' : ''} ${value === option.value ? 'bg-accent/12 text-accent' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'} has-disabled:cursor-not-allowed has-disabled:opacity-40`}>
            <input
                class="absolute inset-0 size-full cursor-pointer appearance-none opacity-0 disabled:cursor-not-allowed"
                type="radio"
                {name}
                value={option.value}
                bind:group={value}
                disabled={option.disabled}
                onchange={() => onchange?.(option.value)} />
            <span class="max-w-full truncate">{option.label}</span>
        </label>
    {/each}
</div>

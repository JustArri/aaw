<script lang="ts">
    type Props = {
        label: string;
        value: number;
        unit: string;
        smallStep?: number;
        largeStep?: number;
        resetValue?: number;
        min?: number;
        max?: number;
        precision?: number;
        disabled?: boolean;
        wideValue?: boolean;
        compact?: boolean;
        onchange?: (value: number) => void | number | Promise<void | number>;
    };

    let {
        label,
        value,
        unit,
        smallStep = 1,
        largeStep = 10,
        resetValue = 0,
        min = Number.NEGATIVE_INFINITY,
        max = Number.POSITIVE_INFINITY,
        precision = 0,
        disabled = false,
        wideValue = false,
        compact = false,
        onchange
    }: Props = $props();

    let displayedValue = $state(0);
    let pending = $state(false);
    let failed = $state(false);

    $effect(() => {
        displayedValue = value;
    });

    function clamp(next: number) {
        return Math.min(max, Math.max(min, next));
    }

    function format(next: number) {
        return next
            .toFixed(precision)
            .replace(/^-0(?:\.0+)?$/, precision ? `0.${'0'.repeat(precision)}` : '0');
    }

    async function commit(next: number) {
        if (disabled || pending || !Number.isFinite(next)) return;

        const previous = value;
        const normalized = clamp(Number(next.toFixed(precision)));
        displayedValue = normalized;
        failed = false;
        pending = true;

        try {
            const authoritativeValue = await onchange?.(normalized);
            if (typeof authoritativeValue === 'number') displayedValue = clamp(authoritativeValue);
        } catch {
            displayedValue = previous;
            failed = true;
        } finally {
            pending = false;
        }
    }

    const actionLabel = (amount: number) =>
        `${amount > 0 ? 'Increase' : 'Decrease'} ${label} by ${format(Math.abs(amount))} ${unit}`;
</script>

<section
    class={`transition-[border-color,opacity] duration-150 ${compact ? 'border-0 bg-transparent p-0' : 'bg-surface-card rounded-xl border p-3.5 min-[37.5rem]:p-4'} ${failed ? 'border-danger-border' : compact ? '' : 'border-border'} ${pending ? 'opacity-70' : ''}`}
    aria-busy={pending}>
    <header
        class={`flex items-center justify-between gap-2 ${compact ? 'mb-1.5 min-h-8' : 'mb-2 min-h-9'}`}>
        <div class="min-w-0">
            <label
                class={`block ${compact ? 'text-foreground-muted text-sm font-normal' : 'text-foreground-strong text-sm font-bold tracking-[.06em] uppercase min-[56.25rem]:text-sm'}`}
                for={`${label}-numeric-input`}>
                {label}
            </label>
            {#if failed}<span
                    class="text-danger-foreground mt-0.5 block text-xs min-[56.25rem]:text-sm"
                    role="status">
                    Update failed
                </span>{/if}
        </div>
        <div
            class="border-border-strong bg-surface-control focus-within:border-accent/65 grid shrink-0 grid-cols-[minmax(0,1fr)_auto] items-stretch overflow-hidden rounded-lg border"
            class:w-40={wideValue}
            class:w-37={!wideValue}>
            <input
                class={`text-foreground h-9 min-w-0 border-0 bg-transparent px-2.5 font-mono font-semibold outline-0 ${compact ? 'text-sm' : 'text-base min-[56.25rem]:text-lg'}`}
                id={`${label}-numeric-input`}
                type="number"
                value={format(displayedValue)}
                step={smallStep}
                {min}
                {max}
                {disabled}
                onchange={(event) => commit(event.currentTarget.valueAsNumber)} />
            <span
                class={`border-border text-foreground-muted flex items-center justify-center border-l px-2 whitespace-nowrap ${compact ? 'text-[0.625rem]' : 'text-[0.6875rem]'}`}>
                {unit}
            </span>
        </div>
    </header>

    <div class="grid grid-cols-5 gap-1.5">
        <button
            class={`border-border-strong bg-surface-hover text-foreground-strong active:border-accent/45 active:bg-accent/12 active:text-accent min-w-0 cursor-pointer rounded-lg border px-1 font-mono disabled:cursor-wait disabled:opacity-50 ${compact ? 'min-h-9 text-xs' : 'min-h-11 text-sm min-[56.25rem]:text-sm'}`}
            type="button"
            disabled={disabled || pending}
            aria-label={actionLabel(-largeStep)}
            onclick={() => commit(displayedValue - largeStep)}>
            −{format(largeStep)}
        </button>
        <button
            class={`border-border-strong bg-surface-hover text-foreground-strong active:border-accent/45 active:bg-accent/12 active:text-accent min-w-0 cursor-pointer rounded-lg border px-1 font-mono disabled:cursor-wait disabled:opacity-50 ${compact ? 'min-h-9 text-xs' : 'min-h-11 text-sm min-[56.25rem]:text-sm'}`}
            type="button"
            disabled={disabled || pending}
            aria-label={actionLabel(-smallStep)}
            onclick={() => commit(displayedValue - smallStep)}>
            −{format(smallStep)}
        </button>
        <button
            class={`border-border-strong bg-surface-hover text-accent active:border-accent/45 active:bg-accent/12 min-w-0 cursor-pointer rounded-lg border px-1 font-bold uppercase disabled:cursor-wait disabled:opacity-50 ${compact ? 'min-h-9 text-[0.625rem]' : 'min-h-11 text-xs min-[56.25rem]:text-sm'}`}
            type="button"
            disabled={disabled || pending}
            aria-label={`Reset ${label} to ${format(resetValue)} ${unit}`}
            onclick={() => commit(resetValue)}>
            Reset
        </button>
        <button
            class={`border-border-strong bg-surface-hover text-foreground-strong active:border-accent/45 active:bg-accent/12 active:text-accent min-w-0 cursor-pointer rounded-lg border px-1 font-mono disabled:cursor-wait disabled:opacity-50 ${compact ? 'min-h-9 text-xs' : 'min-h-11 text-sm min-[56.25rem]:text-sm'}`}
            type="button"
            disabled={disabled || pending}
            aria-label={actionLabel(smallStep)}
            onclick={() => commit(displayedValue + smallStep)}>
            +{format(smallStep)}
        </button>
        <button
            class={`border-border-strong bg-surface-hover text-foreground-strong active:border-accent/45 active:bg-accent/12 active:text-accent min-w-0 cursor-pointer rounded-lg border px-1 font-mono disabled:cursor-wait disabled:opacity-50 ${compact ? 'min-h-9 text-xs' : 'min-h-11 text-sm min-[56.25rem]:text-sm'}`}
            type="button"
            disabled={disabled || pending}
            aria-label={actionLabel(largeStep)}
            onclick={() => commit(displayedValue + largeStep)}>
            +{format(largeStep)}
        </button>
    </div>
</section>

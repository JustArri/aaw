<script lang="ts">
    let {
        label,
        checked = $bindable(false),
        disabled = false,
        indicatorEnd = false,
        class: className = '',
        onchange
    }: {
        label: string;
        checked?: boolean;
        disabled?: boolean;
        indicatorEnd?: boolean;
        class?: string;
        onchange?: (checked: boolean) => void;
    } = $props();

    function toggle() {
        if (disabled) {
            return;
        }
        checked = !checked;
        onchange?.(checked);
    }
</script>

{#snippet indicator()}
    <span
        class={`grid size-4.5 shrink-0 place-items-center rounded-full border transition-colors ${checked ? 'border-accent/70 bg-accent/10' : 'border-border-strong bg-surface-card'}`}
        aria-hidden="true">
        {#if checked}<span class="bg-accent size-2 rounded-full"></span>{/if}
    </span>
{/snippet}

<button
    class={`flex min-h-9 cursor-pointer items-center gap-2 rounded-lg border px-2.5 text-left text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${checked ? 'border-accent/40 bg-accent/6 text-foreground' : 'border-border-strong bg-surface-inset text-foreground hover:border-accent/35'} ${className}`}
    type="button"
    aria-pressed={checked}
    {disabled}
    onclick={toggle}>
    {#if !indicatorEnd}{@render indicator()}{/if}
    <span class="truncate">{label}</span>
    {#if indicatorEnd}{@render indicator()}{/if}
</button>

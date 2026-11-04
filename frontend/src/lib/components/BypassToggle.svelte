<script lang="ts">
    import Icon from '@iconify/svelte';

    let {
        checked = $bindable(false),
        saving = false,
        onchange
    }: {
        checked?: boolean;
        saving?: boolean;
        onchange?: (checked: boolean) => void;
    } = $props();
</script>

<label
    class="text-foreground-muted has-focus-visible:outline-accent relative flex cursor-pointer items-center gap-2 rounded-md text-xs font-medium has-focus-visible:outline-2 has-focus-visible:outline-offset-2">
    <span>Bypass</span>
    <input
        class="absolute inset-0 size-full cursor-pointer appearance-none opacity-0 disabled:cursor-not-allowed"
        type="checkbox"
        bind:checked
        disabled={saving}
        onchange={() => onchange?.(checked)} />
    <span
        class={`relative h-4 w-7 shrink-0 rounded-full border transition-colors ${checked ? 'border-accent/60 bg-accent/25' : 'border-border-strong bg-surface-inset'}`}
        aria-hidden="true">
        {#if saving}
            <Icon
                icon="lucide:loader-circle"
                width="0.75rem"
                class="text-accent absolute inset-0 m-auto animate-spin" />
        {:else}<span
                class={`absolute top-0.5 left-0.5 size-2.5 rounded-full transition-all ${checked ? 'bg-accent translate-x-3' : 'bg-foreground-icon'}`}>
            </span>{/if}
    </span>
</label>

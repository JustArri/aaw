<script lang="ts">
    import Icon from '@iconify/svelte';

    let {
        saving = false,
        pending = false,
        disabled = false,
        label = 'Apply changes'
    }: {
        saving?: boolean;
        pending?: boolean;
        disabled?: boolean;
        label?: string;
    } = $props();
</script>

<button
    class={`grid size-9 shrink-0 place-items-center rounded-lg border-0 bg-transparent transition-colors ${saving ? 'text-accent cursor-wait' : disabled || !pending ? 'text-foreground-icon cursor-default opacity-50' : 'text-accent hover:bg-accent/10 cursor-pointer'}`}
    type="submit"
    aria-label={label}
    title={label}
    aria-busy={saving}
    disabled={disabled || saving || !pending}>
    <Icon
        icon={saving ? 'lucide:loader-circle' : 'lucide:check'}
        width="1.125rem"
        class={saving ? 'animate-spin' : ''} />
</button>

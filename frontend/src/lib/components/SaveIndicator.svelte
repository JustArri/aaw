<script lang="ts">
    import Icon from '@iconify/svelte';

    let {
        saving = false,
        pending = false
    }: {
        saving?: boolean;
        pending?: boolean;
    } = $props();

    let label = $derived(saving ? 'Saving…' : pending ? 'Pending change' : 'Saved');
</script>

<span
    class={`grid size-9 shrink-0 place-items-center ${saving || pending ? 'text-accent' : 'text-foreground-icon'}`}
    title={label}
    aria-label={label}>
    <Icon
        icon={saving ? 'lucide:loader-circle' : 'lucide:check'}
        width="1.125rem"
        class={saving ? 'animate-spin' : ''} />
</span>

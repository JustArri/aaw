<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';

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

<Button
    variant="accent"
    size="sm"
    class="justify-self-end"
    type="submit"
    aria-label={label}
    title={label}
    aria-busy={saving}
    busy={saving}
    disabled={disabled || saving || !pending}>
    {#if saving}
        <Icon icon="lucide:loader-circle" width="0.875rem" class="animate-spin" />
    {/if}
    {saving ? 'Applying…' : 'Apply'}
</Button>

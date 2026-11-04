<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';

    interface Props {
        title: string;
        description: string;
        loading: boolean;
        saving: boolean;
        dirty: boolean;
        onadd: () => void;
    }

    let { title, description, loading, saving, dirty, onadd }: Props = $props();
</script>

<header
    class="border-border flex min-h-16.75 items-center justify-between gap-3 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
    <div>
        <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
            {title}
        </h2>
        <p class="text-foreground-muted mt-1 mb-0 text-sm">{description}</p>
    </div>
    <Button
        variant="accent-outline"
        size="sm"
        busy={saving}
        disabled={loading || saving || dirty}
        onclick={onadd}>
        <Icon icon="lucide:plus" width="0.875rem" /> Add
    </Button>
</header>

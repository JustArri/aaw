<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import { recordings, type RecordingsSection } from '$lib/sections/recordings.svelte';

    interface Props {
        section: RecordingsSection;
    }

    let { section }: Props = $props();
</script>

<header
    class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
    <div>
        <h2 class="m-0 text-base font-semibold min-[56.25rem]:text-lg">Recordings</h2>
        <p class="text-foreground-muted mt-1 mb-0 text-sm">
            {recordings().length}
            {recordings().length === 1 ? 'file' : 'files'} available
        </p>
    </div>
    <Button
        variant="secondary"
        size="lg"
        square
        busy={section.refreshing}
        disabled={section.refreshing}
        aria-label="Refresh recordings"
        title="Refresh recordings"
        onclick={() => void section.refresh()}>
        <Icon
            class={section.refreshing ? 'animate-spin' : ''}
            icon={section.refreshing ? 'lucide:loader-circle' : 'lucide:refresh-cw'}
            width="1rem" />
    </Button>
</header>

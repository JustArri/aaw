<script lang="ts">
    import Icon from '@iconify/svelte';
    import { recordings, type RecordingsSection } from '$lib/sections/recordings.svelte';
    import RecordingListItem from './RecordingListItem.svelte';

    interface Props {
        section: RecordingsSection;
    }

    let { section }: Props = $props();
</script>

{#if section.loading}
    <div class="text-foreground-muted grid min-h-48 place-items-center">
        <Icon class="animate-spin" icon="lucide:loader-circle" width="1.5rem" />
    </div>
{:else if recordings().length > 0}
    <div class="divide-border divide-y">
        {#each recordings() as recording (recording.path)}
            <RecordingListItem {section} {recording} />
        {/each}
    </div>
{:else}
    <div
        class="text-foreground-muted grid min-h-56 place-items-center content-center px-6 text-center">
        <span class="bg-accent/10 text-accent grid size-12 place-items-center rounded-xl">
            <Icon icon="lucide:file-music" width="1.5rem" />
        </span>
        <h3 class="text-foreground mt-3 mb-0 text-sm font-semibold">No recordings</h3>
        <p class="mt-1.5 mb-0 max-w-72 text-sm">
            Finished audio recordings will appear here for download or deletion.
        </p>
    </div>
{/if}

<script lang="ts">
    import type { RecordingsSection } from '$lib/sections/recordings.svelte';
    import RecordingsHeader from './RecordingsHeader.svelte';
    import RecordingsList from './RecordingsList.svelte';

    interface Props {
        section: RecordingsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <RecordingsHeader {section} />

    {#if section.errorMessage}
        <div class="border-danger/35 bg-danger/8 text-danger border-b px-3.75 py-3 text-sm">
            {section.errorMessage}
        </div>
    {/if}

    <RecordingsList {section} />
</section>

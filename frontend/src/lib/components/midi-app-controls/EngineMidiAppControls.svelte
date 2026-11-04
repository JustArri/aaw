<script lang="ts">
    import type { MidiAppControlsSection } from '$lib/sections/midi-app-controls.svelte';
    import AppControlsHeader from './AppControlsHeader.svelte';
    import EmptyMappings from './EmptyMappings.svelte';
    import EngineMidiControlForm from './EngineMidiControlForm.svelte';

    interface Props {
        section: MidiAppControlsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <AppControlsHeader
        title="Engine MIDI app controls"
        description="Global mappings loaded at startup and applied to every project."
        loading={section.loading}
        saving={section.saving}
        dirty={section.dirty.size > 0}
        onadd={() => void section.addControl()} />

    {#if section.loading}
        <div class="text-foreground-muted grid min-h-40 place-items-center text-sm">
            Loading MIDI app controls…
        </div>
    {:else if section.drafts.length === 0}
        <EmptyMappings
            title="No engine mappings"
            description="Add a mapping to control transport from an external MIDI device."
            saving={section.saving}
            onadd={() => void section.addControl()} />
    {:else}
        <div class="grid gap-3 p-3.75 min-[56.25rem]:p-5">
            {#each section.drafts as control, index}
                <EngineMidiControlForm {section} {control} {index} />
            {/each}
        </div>
    {/if}
</section>

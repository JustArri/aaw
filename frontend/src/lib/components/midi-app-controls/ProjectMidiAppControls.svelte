<script lang="ts">
    import type { ProjectMidiAppControlsSection } from '$lib/sections/project-midi-app-controls.svelte';
    import AppControlsHeader from './AppControlsHeader.svelte';
    import EmptyMappings from './EmptyMappings.svelte';
    import ProjectMidiControlForm from './ProjectMidiControlForm.svelte';

    interface Props {
        section: ProjectMidiAppControlsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <AppControlsHeader
        title="Project MIDI app controls"
        description="Mappings saved with the current project."
        loading={section.loading}
        saving={section.saving}
        dirty={section.dirty.size > 0}
        onadd={() => void section.addControl()} />

    {#if section.loading}
        <div class="text-foreground-muted grid min-h-40 place-items-center text-sm">Loading…</div>
    {:else if section.drafts.length === 0}
        <EmptyMappings
            title="No project mappings"
            description="Add a mapping to control this project from a MIDI device."
            saving={section.saving}
            onadd={() => void section.addControl()} />
    {:else}
        <div class="grid gap-3 p-3.75 min-[56.25rem]:p-5">
            {#each section.drafts as control, index}
                <ProjectMidiControlForm {section} {control} {index} />
            {/each}
        </div>
    {/if}
</section>

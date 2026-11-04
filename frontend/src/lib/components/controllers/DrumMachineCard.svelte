<script lang="ts">
    import type { ControllersSection, Machine } from '$lib/sections/controllers.svelte';
    import DrumMachineGrid from './DrumMachineGrid.svelte';
    import DrumMachinePresets from './DrumMachinePresets.svelte';
    import DrumMachineToolbar from './DrumMachineToolbar.svelte';
    import StepEditor from './StepEditor.svelte';
    import VoiceEditor from './VoiceEditor.svelte';

    interface Props {
        section: ControllersSection;
        machine: Machine;
    }

    let { section, machine }: Props = $props();
</script>

<article class="border-border bg-surface-panel grid min-w-0 gap-3 rounded-xl border p-3">
    <DrumMachineToolbar {section} {machine} />
    <DrumMachinePresets {section} {machine} />
    <DrumMachineGrid {section} {machine} />

    {#if section.selectedVoiceLane(machine)}
        <VoiceEditor {section} {machine} lane={section.selectedVoiceLane(machine)!} />
    {/if}

    {#if section.selectedStepValue(machine) && section.selectedStep?.machineId === machine.id}
        <StepEditor {section} {machine} />
    {/if}

    {#if section.saving.has(machine.id)}
        <p class="text-foreground-muted m-0 text-right text-xs">Saving…</p>
    {/if}
</article>

<script lang="ts">
    import Icon from '@iconify/svelte';
    import AutoConnectionsPanel from '$lib/components/midi-inputs/AutoConnectionsPanel.svelte';
    import InputSlotsPanel from '$lib/components/midi-inputs/InputSlotsPanel.svelte';
    import { MidiInputsSection } from '$lib/sections/midi-inputs.svelte';

    const section = new MidiInputsSection();
</script>

{#if section.errorMessage}
    <div
        class="border-danger/30 bg-danger/10 text-danger-foreground mb-3 flex items-start gap-2 rounded-xl border px-3.75 py-3 text-sm"
        role="alert">
        <Icon icon="lucide:circle-alert" width="1rem" class="mt-px shrink-0" />
        <span>{section.errorMessage}</span>
    </div>
{/if}

<div class="grid min-w-0 gap-3 min-[56.25rem]:grid-cols-[minmax(0,1.15fr)_minmax(20rem,0.85fr)]">
    <InputSlotsPanel {section} />
    <AutoConnectionsPanel {section} />
</div>

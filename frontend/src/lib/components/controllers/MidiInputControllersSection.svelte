<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { ControllersSection } from '$lib/sections/controllers.svelte';
    import MidiInputControllersPanel from '$lib/components/instruments/MidiInputControllersPanel.svelte';

    interface Props {
        section: ControllersSection;
    }

    let { section }: Props = $props();
</script>

<section class="grid gap-3">
    <div>
        <h3 class="text-foreground m-0 text-sm font-semibold">MIDI input controllers</h3>
        <p class="text-foreground-muted m-0 text-xs">Live MIDI routing and transforms</p>
    </div>
    {#key section.controllerEditorRevision}
        <MidiInputControllersPanel
            embedded
            instrumentId={section.instrumentId}
            showAddButton={false} />
    {/key}
    <Button
        class="justify-self-end"
        variant="accent-outline"
        busy={section.addingMidiController}
        onclick={() => section.addMidiController()}>
        <Icon icon="lucide:plus" width="1rem" /> Add
    </Button>
</section>

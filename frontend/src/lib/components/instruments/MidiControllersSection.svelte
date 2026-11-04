<script lang="ts">
    import ApplyButton from '$lib/components/ApplyButton.svelte';
    import SegmentedControl from '$lib/components/SegmentedControl.svelte';
    import {
        type InstrumentsSection,
        MidiFilterKind,
        submitControlGroupOnEnter
    } from '$lib/sections/instruments.svelte';
    import MidiControllerPicker from './MidiControllerPicker.svelte';
    import SelectedControllers from './SelectedControllers.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<form
    class="border-border grid gap-5 border-b p-4"
    use:submitControlGroupOnEnter
    onsubmit={(event) => {
        event.preventDefault();
        void section.controllerSection.save();
    }}>
    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">MIDI controllers</h3>
    <div class="grid gap-2">
        <div class="grid min-w-0 gap-4">
            <SegmentedControl
                name="midi-controller-filter-mode"
                ariaLabel="MIDI controller filter mode"
                compact
                bind:value={section.controllerSection.type}
                options={[
                    { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                    { value: MidiFilterKind.Denylist, label: 'Allow all except' }
                ]} />
            <MidiControllerPicker {section} />
            <SelectedControllers {section} />
        </div>
        <ApplyButton
            saving={section.savingSections.has('controllers')}
            pending={section.controllerSection.changed} />
    </div>
    <small class="text-foreground-muted">
        Selected controllers are {section.controllerSection.type === MidiFilterKind.Allowlist
            ? 'allowed'
            : 'blocked'}.
    </small>
</form>

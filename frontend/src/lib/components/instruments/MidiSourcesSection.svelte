<script lang="ts">
    import ApplyButton from '$lib/components/ApplyButton.svelte';
    import NumberedCheckboxGrid from '$lib/components/NumberedCheckboxGrid.svelte';
    import SegmentedControl from '$lib/components/SegmentedControl.svelte';
    import {
        type InstrumentsSection,
        MidiFilterKind,
        submitControlGroupOnEnter
    } from '$lib/sections/instruments.svelte';

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
        void section.sourceSection.save();
    }}>
    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">MIDI input sources</h3>
    <div class="grid gap-2">
        <div class="grid min-w-0 gap-4">
            <SegmentedControl
                name="midi-source-filter-mode"
                ariaLabel="MIDI input source filter mode"
                compact
                bind:value={section.sourceSection.type}
                options={[
                    { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                    { value: MidiFilterKind.Denylist, label: 'Allow all except' }
                ]} />
            <NumberedCheckboxGrid
                values={section.midiSources}
                selected={section.sourceSection.values}
                itemLabel="MIDI input source"
                ontoggle={(value, checked) => section.sourceSection.toggle(value, checked)}
                onsetall={(values) => (section.sourceSection.values = [...values])} />
        </div>
        <ApplyButton
            saving={section.savingSections.has('sources')}
            pending={section.sourceSection.changed} />
    </div>
    <small class="text-foreground-muted">
        Checked input slots are {section.sourceSection.type === MidiFilterKind.Allowlist
            ? 'allowed'
            : 'blocked'}.
    </small>
</form>

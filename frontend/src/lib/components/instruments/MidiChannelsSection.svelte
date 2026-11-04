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
        void section.channelSection.save();
    }}>
    <h3 class="text-foreground-strong m-0 text-xs font-semibold uppercase">MIDI channels</h3>
    <div class="grid gap-2">
        <div class="grid min-w-0 gap-4">
            <SegmentedControl
                name="midi-channel-filter-mode"
                ariaLabel="MIDI channel filter mode"
                compact
                bind:value={section.channelSection.type}
                options={[
                    { value: MidiFilterKind.Allowlist, label: 'Allow only' },
                    { value: MidiFilterKind.Denylist, label: 'Allow all except' }
                ]} />
            <NumberedCheckboxGrid
                values={section.midiChannels}
                selected={section.channelSection.values}
                itemLabel="MIDI channel"
                ontoggle={(value, checked) => section.channelSection.toggle(value, checked)}
                onsetall={(values) => (section.channelSection.values = [...values])} />
        </div>
        <ApplyButton
            saving={section.savingSections.has('channels')}
            pending={section.channelSection.changed} />
    </div>
    <small class="text-foreground-muted">
        Checked channels are {section.channelSection.type === MidiFilterKind.Allowlist
            ? 'allowed'
            : 'blocked'}.
    </small>
</form>

<script lang="ts">
    import type { CtrMidiEngineControl } from '$lib/generated/models';
    import type { MidiAppControlsSection } from '$lib/sections/midi-app-controls.svelte';
    import ControlIdentity from './ControlIdentity.svelte';
    import ControlStatus from './ControlStatus.svelte';
    import EngineControlFields from './EngineControlFields.svelte';
    import MidiFilterFieldset from './MidiFilterFieldset.svelte';

    interface Props {
        section: MidiAppControlsSection;
        control: CtrMidiEngineControl;
        index: number;
    }

    let { section, control, index }: Props = $props();
</script>

<form
    class="border-border bg-surface-inset grid gap-4 rounded-xl border p-3.5"
    onsubmit={(event) => {
        event.preventDefault();
        void section.applyControl();
    }}>
    <ControlIdentity
        name={control.name}
        {index}
        learning={section.learningIndex === index}
        saving={section.saving}
        removeDisabled={section.dirty.size > 0}
        onnamechange={(value) => {
            control.name = value;
            section.markDirty(index);
        }}
        ontogglelearning={() =>
            (section.learningIndex = section.learningIndex === index ? null : index)}
        onremove={() => section.removeControl(index)} />

    <EngineControlFields {section} {control} {index} />

    <div class="grid gap-4 min-[56.25rem]:grid-cols-2">
        <MidiFilterFieldset
            filter={control.source_filter}
            values={section.midiSources}
            name={`source-filter-${index}`}
            ariaLabel={`Mapping ${index + 1} source filter mode`}
            legend="MIDI input sources"
            checkedLabel="slots"
            description={section.filterDescription(control.source_filter)}
            onmodechange={(value) => section.setFilterMode(index, 'source_filter', value)}
            ontoggle={(value, checked) =>
                section.toggleFilterValue(index, 'source_filter', value, checked)}
            onsetall={(values) => section.setAllFilterValues(index, 'source_filter', values)} />
        <MidiFilterFieldset
            filter={control.channel_filter}
            values={section.midiChannels}
            name={`channel-filter-${index}`}
            ariaLabel={`Mapping ${index + 1} channel filter mode`}
            legend="MIDI channels"
            checkedLabel="channels"
            description={section.filterDescription(control.channel_filter)}
            onmodechange={(value) => section.setFilterMode(index, 'channel_filter', value)}
            ontoggle={(value, checked) =>
                section.toggleFilterValue(index, 'channel_filter', value, checked)}
            onsetall={(values) => section.setAllFilterValues(index, 'channel_filter', values)} />
    </div>

    <ControlStatus
        message={section.learningIndex === index
            ? 'Move a CC control or play a note on the desired MIDI source.'
            : section.dirty.has(index)
              ? 'This mapping has unapplied changes.'
              : `Mapping ${index + 1} is synchronized with the engine.`}
        saving={section.saving}
        pending={section.dirty.has(index)}
        applyLabel="Apply all MIDI app control changes" />
</form>

<script lang="ts">
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import type { CtrMidiEngineControl } from '$lib/generated/models';
    import type { MidiAppControlsSection } from '$lib/sections/midi-app-controls.svelte';

    interface Props {
        section: MidiAppControlsSection;
        control: CtrMidiEngineControl;
        index: number;
    }

    let { section, control, index }: Props = $props();
</script>

<div class="grid gap-3 min-[37.5rem]:grid-cols-2">
    <label class="grid min-w-0 gap-1">
        <span class="text-foreground-muted text-xs">Action</span>
        <SelectDropdown
            value={section.controlType(control)}
            options={section.controlTypes}
            ariaLabel={`Mapping ${index + 1} action`}
            onchange={(value) => section.changeType(index, value)} />
    </label>
    {#if section.isNoteControl(control)}
        <NumericControl
            compact
            label="MIDI note"
            value={section.noteKey(control)}
            unit="note"
            smallStep={1}
            largeStep={12}
            resetValue={60}
            min={0}
            max={127}
            onchange={(value) => {
                section.setNoteKey(control, value);
                section.markDirty(index);
            }} />
    {:else if 'CCTransportSwitch' in control.kind}
        <div class="grid gap-3 min-[37.5rem]:grid-cols-2">
            <NumericControl
                compact
                label="MIDI CC"
                value={control.kind.CCTransportSwitch.controller}
                unit="CC"
                smallStep={1}
                largeStep={10}
                resetValue={64}
                min={0}
                max={127}
                onchange={(value) => {
                    if ('CCTransportSwitch' in control.kind) {
                        control.kind.CCTransportSwitch.controller = value;
                        section.markDirty(index);
                    }
                }} />
            <NumericControl
                compact
                label="On threshold"
                value={control.kind.CCTransportSwitch.threshold}
                unit="MIDI"
                smallStep={1}
                largeStep={10}
                resetValue={64}
                min={0}
                max={127}
                onchange={(value) => {
                    if ('CCTransportSwitch' in control.kind) {
                        control.kind.CCTransportSwitch.threshold = value;
                        section.markDirty(index);
                    }
                }} />
        </div>
    {/if}
</div>

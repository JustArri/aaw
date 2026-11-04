<script lang="ts">
    import NumericControl from '$lib/components/NumericControl.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import type { CtrMidiProjectControl } from '$lib/generated/models';
    import type { ProjectMidiAppControlsSection } from '$lib/sections/project-midi-app-controls.svelte';

    interface Props {
        section: ProjectMidiAppControlsSection;
        control: CtrMidiProjectControl;
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
    <label class="grid min-w-0 gap-1">
        <span class="text-foreground-muted text-xs">
            {section.isControllerControl(control) ? 'Instrument controller' : 'User preset'}
        </span>
        {#if section.isControllerControl(control)}
            <SelectDropdown
                value={section.controllerTargetValue(control)}
                options={section.controllerOptionsFor(control)}
                ariaLabel={`Mapping ${index + 1} instrument controller`}
                onchange={(value) => section.setControllerTarget(index, value)} />
        {:else}
            <SelectDropdown
                value={String(section.presetIndex(control))}
                options={section.presetOptionsFor(control)}
                ariaLabel={`Mapping ${index + 1} user preset`}
                onchange={(value) => section.setPreset(index, value)} />
        {/if}
    </label>
</div>

<div class="grid gap-3 min-[37.5rem]:grid-cols-2">
    <NumericControl
        compact
        label={section.isNoteControl(control) ? 'MIDI note' : 'MIDI CC'}
        value={section.isNoteControl(control)
            ? section.noteKey(control)
            : section.ccNumber(control)}
        unit={section.isNoteControl(control) ? 'note' : 'CC'}
        smallStep={1}
        largeStep={section.isNoteControl(control) ? 12 : 10}
        resetValue={section.isNoteControl(control) ? 60 : 64}
        min={0}
        max={127}
        onchange={(value) => {
            section.setTrigger(control, value);
            section.markDirty(index);
        }} />
    {#if 'CCInstrumentControllerSwitch' in control.kind}
        <NumericControl
            compact
            label="On threshold"
            value={control.kind.CCInstrumentControllerSwitch.threshold}
            unit="MIDI"
            smallStep={1}
            largeStep={10}
            resetValue={64}
            min={0}
            max={127}
            onchange={(value) => {
                if ('CCInstrumentControllerSwitch' in control.kind) {
                    control.kind.CCInstrumentControllerSwitch.threshold = value;
                    section.markDirty(index);
                }
            }} />
    {/if}
</div>

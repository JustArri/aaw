<script lang="ts">
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import type { DrumLane } from '$lib/generated/models';
    import type { ControllersSection, Machine } from '$lib/sections/controllers.svelte';

    interface Props {
        section: ControllersSection;
        machine: Machine;
        lane: DrumLane;
    }

    let { section, machine, lane }: Props = $props();
</script>

<div
    bind:this={section.voiceEditor}
    class={`border-border grid gap-3 rounded-lg border-t pt-3 transition-[background-color,box-shadow] min-[37.5rem]:grid-cols-2 min-[56.25rem]:grid-cols-3 ${section.voiceEditorHighlighted ? 'bg-accent/10 ring-accent/50 animate-[pulse_900ms_ease-out_1] ring-2' : ''}`}>
    <div class="grid content-start gap-1">
        <label
            class="text-foreground-muted text-sm"
            for={`drum-machine-${machine.id}-lane-${lane.id}-name`}>
            Lane name
        </label>
        <AutoSaveTextInput
            id={`drum-machine-${machine.id}-lane-${lane.id}-name`}
            bind:value={lane.name}
            pending={lane.name !== section.selectedVoice?.initialName}
            saving={section.saving.has(machine.id)}
            oncommit={async () => {
                if (
                    (await section.saveMachine(machine)) &&
                    section.selectedVoice?.machineId === machine.id &&
                    section.selectedVoice.laneId === lane.id
                ) {
                    section.selectedVoice.initialName = lane.name;
                }
            }} />
    </div>
    <NumericControl
        compact
        label={`${lane.name} MIDI note`}
        value={lane.output_note}
        unit="note"
        smallStep={1}
        largeStep={12}
        resetValue={section.selectedVoice?.initialNote ?? lane.output_note}
        min={0}
        max={127}
        onchange={async (value) => {
            lane.output_note = value;
            await section.saveMachine(machine);
            return lane.output_note;
        }} />
    <NumericControl
        compact
        label={`${lane.name} MIDI channel`}
        value={lane.output_channel + 1}
        unit="channel"
        smallStep={1}
        largeStep={4}
        resetValue={section.selectedVoice?.initialChannel ?? lane.output_channel + 1}
        min={1}
        max={16}
        onchange={async (value) => {
            lane.output_channel = value - 1;
            await section.saveMachine(machine);
            return lane.output_channel + 1;
        }} />
    <NumericControl
        compact
        label={`${lane.name} velocity`}
        value={lane.velocity}
        unit="MIDI"
        smallStep={1}
        largeStep={10}
        resetValue={section.selectedVoice?.initialVelocity ?? lane.velocity}
        min={1}
        max={127}
        onchange={async (value) => {
            lane.velocity = value;
            await section.saveMachine(machine);
            return lane.velocity;
        }} />
    <div class="flex items-end justify-end min-[37.5rem]:col-span-2 min-[56.25rem]:col-span-3">
        <TimedConfirmButton
            label="Remove lane"
            confirmLabel="Confirm remove"
            busyLabel="Removing"
            busy={section.saving.has(machine.id)}
            onconfirm={() => section.removeLane(machine, lane)} />
    </div>
</div>

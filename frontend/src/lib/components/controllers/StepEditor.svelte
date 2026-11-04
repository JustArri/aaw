<script lang="ts">
    import NumericControl from '$lib/components/NumericControl.svelte';
    import type { ControllersSection, Machine } from '$lib/sections/controllers.svelte';

    interface Props {
        section: ControllersSection;
        machine: Machine;
    }

    let { section, machine }: Props = $props();
    const step = $derived(section.selectedStepValue(machine)!);
    const lane = $derived(section.selectedLane(machine)!);
</script>

<div class="border-border flex flex-wrap items-end gap-3 border-t pt-3">
    <span class="text-foreground-muted mr-auto text-xs">
        {lane.name} · Step {section.selectedStep!.index + 1}
    </span>
    <div class="grid w-full gap-3 min-[37.5rem]:grid-cols-2">
        <NumericControl
            compact
            label="Step velocity"
            value={step.velocity}
            unit="MIDI"
            smallStep={1}
            largeStep={10}
            resetValue={100}
            min={1}
            max={127}
            onchange={async (value) => {
                step.velocity = value;
                await section.saveMachine(machine);
                return step.velocity;
            }} />
        <NumericControl
            compact
            label="Gate length"
            value={step.gate_length_ticks}
            unit="ticks"
            smallStep={10}
            largeStep={120}
            resetValue={120}
            min={1}
            max={960}
            onchange={async (value) => {
                step.gate_length_ticks = value;
                await section.saveMachine(machine);
                return step.gate_length_ticks;
            }} />
    </div>
</div>

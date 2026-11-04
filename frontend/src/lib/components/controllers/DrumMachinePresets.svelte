<script lang="ts">
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import type { ControllersSection, Machine } from '$lib/sections/controllers.svelte';

    interface Props {
        section: ControllersSection;
        machine: Machine;
    }

    let { section, machine }: Props = $props();
</script>

<div class="grid gap-3 min-[37.5rem]:grid-cols-2">
    <div class="flex min-w-0 items-end gap-2">
        <label class="grid min-w-0 flex-1 gap-1">
            <span class="text-foreground-muted text-xs">Lane preset</span>
            <SelectDropdown
                value={section.selectedLanePresets[machine.id] ?? ''}
                options={section.lanePresets.map(({ value, label, description }) => ({
                    value,
                    label,
                    description
                }))}
                placeholder="Choose a drum layout"
                disabled={section.saving.has(machine.id)}
                ariaLabel="Drum-machine lane preset"
                onchange={(value) => {
                    section.selectedLanePresets = {
                        ...section.selectedLanePresets,
                        [machine.id]: value
                    };
                }} />
        </label>
        <TimedConfirmButton
            label="Apply"
            confirmLabel="Replace lanes?"
            busyLabel="Applying"
            busy={section.saving.has(machine.id)}
            disabled={!section.selectedLanePresets[machine.id]}
            onconfirm={() => section.applyLanePreset(machine)} />
    </div>
    <div class="flex min-w-0 items-end gap-2">
        <label class="grid min-w-0 flex-1 gap-1">
            <span class="text-foreground-muted text-xs">Beat preset</span>
            <SelectDropdown
                value={section.selectedBeatPresets[machine.id] ?? ''}
                options={section.beatPresets.map(({ value, label, description }) => ({
                    value,
                    label,
                    description
                }))}
                placeholder="Choose a predefined beat"
                disabled={section.saving.has(machine.id)}
                searchable
                searchPlaceholder="Search beat presets"
                ariaLabel="Drum-machine beat preset"
                onchange={(value) => {
                    section.selectedBeatPresets = {
                        ...section.selectedBeatPresets,
                        [machine.id]: value
                    };
                }} />
        </label>
        <TimedConfirmButton
            label="Apply"
            confirmLabel="Replace lanes and beat?"
            busyLabel="Applying"
            busy={section.saving.has(machine.id)}
            disabled={!section.selectedBeatPresets[machine.id]}
            onconfirm={() => section.applyBeatPreset(machine)} />
    </div>
</div>

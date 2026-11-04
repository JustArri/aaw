<script lang="ts">
    import Icon from '@iconify/svelte';
    import AutoSaveTextInput from '$lib/components/AutoSaveTextInput.svelte';
    import Button from '$lib/components/Button.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import type { ControllersSection, Machine } from '$lib/sections/controllers.svelte';

    interface Props {
        section: ControllersSection;
        machine: Machine;
    }

    let { section, machine }: Props = $props();
</script>

<div class="flex flex-wrap items-end gap-2">
    <div class="grid min-w-48 flex-1 gap-1">
        <label class="text-foreground-muted text-xs" for={`drum-machine-${machine.id}-name`}>
            Name
        </label>
        <AutoSaveTextInput
            id={`drum-machine-${machine.id}-name`}
            bind:value={machine.data.name}
            pending={machine.data.name !== machine.savedName}
            saving={section.saving.has(machine.id)}
            oncommit={() => section.saveMachine(machine)} />
    </div>
    <Button
        size="sm"
        variant="accent-outline"
        disabled={machine.data.lanes.length >= 64 || section.saving.has(machine.id)}
        onclick={() => section.addLane(machine)}>
        <Icon icon="lucide:plus" width="0.875rem" /> Lane
    </Button>
    <TimedConfirmButton
        label="Delete all"
        confirmLabel="Confirm delete"
        busyLabel="Deleting"
        busy={section.saving.has(machine.id)}
        disabled={machine.data.lanes.length === 0}
        onconfirm={() => section.removeAllLanes(machine)} />
    <TimedConfirmButton
        label="Clear all"
        confirmLabel="Confirm clear"
        busyLabel="Clearing"
        disabled={section.saving.has(machine.id) || !section.machineHasSteps(machine)}
        onconfirm={() => section.clearMachine(machine)} />
    <Button
        size="sm"
        variant={machine.data.enabled ? 'accent-outline' : 'secondary'}
        onclick={() => {
            machine.data.enabled = !machine.data.enabled;
            return section.saveMachine(machine);
        }}>
        {machine.data.enabled ? 'Enabled' : 'Disabled'}
    </Button>
    <TimedConfirmButton
        label="Remove"
        confirmLabel="Confirm remove"
        busyLabel="Removing"
        busy={section.removing.has(machine.id)}
        onconfirm={() => section.removeMachine(machine)} />
</div>

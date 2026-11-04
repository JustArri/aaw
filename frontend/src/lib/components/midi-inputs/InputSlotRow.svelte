<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import type { MidiInputSlot } from '$lib/generated/models';
    import type { MidiInputsSection } from '$lib/sections/midi-inputs.svelte';

    interface Props {
        section: MidiInputsSection;
        slot: MidiInputSlot;
        slotId: number;
    }

    let { section, slot, slotId }: Props = $props();
</script>

<article
    class="grid gap-3 px-3.75 py-3.5 min-[37.5rem]:grid-cols-[minmax(7rem,0.45fr)_minmax(0,1fr)_auto] min-[37.5rem]:items-center min-[56.25rem]:px-5">
    <div class="flex min-w-0 items-center gap-3">
        <span
            class={`grid size-8.5 shrink-0 place-items-center rounded-lg ${slot.type === 'Connected' ? 'bg-success/10 text-success' : 'bg-surface-icon text-foreground-muted'}`}>
            <Icon icon="mdi:midi-port" width="1.125rem" />
        </span>
        <div class="min-w-0">
            <strong class="block text-sm">Slot {slotId + 1}</strong>
            <small class="text-foreground-muted mt-0.5 block text-sm">
                {slot.type === 'Connected' ? 'Connected' : 'Available'}
            </small>
        </div>
    </div>

    {#if slot.type === 'Connected'}
        <div class="min-w-0">
            <strong class="block truncate text-sm font-medium">
                {section.portName(slot.data.port_id)}
            </strong>
            <small class="text-foreground-muted mt-0.5 block truncate font-mono text-xs">
                {slot.data.port_id}
            </small>
        </div>
        <Button
            variant="danger-outline"
            size="sm"
            type="button"
            busy={section.busySlotId === slotId}
            disabled={section.busySlotId !== null}
            onclick={() => void section.disconnect(slotId)}>
            {#if section.busySlotId === slotId}
                <Icon icon="lucide:loader-circle" width="0.875rem" class="animate-spin" />
            {/if}
            {section.busySlotId === slotId ? 'Disconnecting…' : 'Disconnect'}
        </Button>
    {:else}
        <SelectDropdown
            bind:value={section.selectedPorts[slotId]}
            options={[
                { value: '', label: 'Select an input port' },
                ...section.ports.map((port) => ({
                    value: port.id,
                    label: port.name,
                    description: section.connectedPortIds.has(port.id) ? 'Connected' : undefined,
                    disabled: section.connectedPortIds.has(port.id)
                }))
            ]}
            ariaLabel={`Port for MIDI input slot ${slotId + 1}`} />
        <Button
            variant="primary"
            size="sm"
            type="button"
            busy={section.busySlotId === slotId}
            disabled={!section.selectedPorts[slotId] || section.busySlotId !== null}
            onclick={() => void section.connect(slotId)}>
            {#if section.busySlotId === slotId}
                <Icon icon="lucide:loader-circle" width="0.875rem" class="animate-spin" />
            {/if}
            {section.busySlotId === slotId ? 'Connecting…' : 'Connect'}
        </Button>
    {/if}
</article>

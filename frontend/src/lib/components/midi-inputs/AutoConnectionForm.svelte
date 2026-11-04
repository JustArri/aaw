<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TextSuggestionInput from '$lib/components/TextSuggestionInput.svelte';
    import type { MidiInputsSection } from '$lib/sections/midi-inputs.svelte';

    interface Props {
        section: MidiInputsSection;
    }

    let { section }: Props = $props();
</script>

<form
    class="border-border grid gap-3 border-b p-3.75 min-[56.25rem]:p-5"
    onsubmit={(event) => {
        event.preventDefault();
        void section.addAutoConnection();
    }}>
    <label class="grid gap-1.5">
        <span class="text-sm font-semibold">Input port</span>
        <TextSuggestionInput
            id="midi-input-port"
            bind:value={section.autoPortId}
            suggestions={section.ports.map((port) => ({
                value: port.id,
                label: port.name,
                description: port.id
            }))}
            placeholder="MIDI port ID"
            clearable
            noMatchesText="No matching ports"
            ariaLabel="MIDI input port" />
        <small class="text-foreground-muted text-xs">
            Enter any port ID or choose a currently available input.
        </small>
    </label>

    <div class="grid gap-3 min-[31.25rem]:grid-cols-2">
        <label class="grid gap-1.5">
            <span class="text-sm font-semibold">Preferred slot</span>
            <SelectDropdown
                id="preferred-midi-slot"
                bind:value={section.preferredSlotId}
                options={[
                    { value: '', label: 'Any available slot' },
                    ...Array.from(section.slots.keys()).map((slotId) => ({
                        value: String(slotId),
                        label: `Slot ${slotId + 1}`
                    }))
                ]}
                onchange={() => {
                    if (section.preferredSlotId === '') section.fixedSlot = false;
                }}
                ariaLabel="Preferred MIDI input slot" />
        </label>

        <label
            class="border-border-strong bg-surface-inset flex min-h-10 cursor-pointer items-center gap-2.5 self-end rounded-lg border px-3 text-sm has-disabled:cursor-not-allowed has-disabled:opacity-50">
            <input
                class="accent-accent size-4"
                type="checkbox"
                bind:checked={section.fixedSlot}
                disabled={section.preferredSlotId === ''} />
            <span>Require preferred slot</span>
        </label>
    </div>

    <Button
        variant="primary"
        size="md"
        class="min-[31.25rem]:w-fit"
        type="submit"
        busy={section.addingAutoConnection}
        disabled={!section.autoPortId.trim() || section.addingAutoConnection}>
        {#if section.addingAutoConnection}
            <Icon icon="lucide:loader-circle" width="0.875rem" class="animate-spin" />
        {/if}
        {section.addingAutoConnection ? 'Adding…' : 'Add auto-connection'}
    </Button>
</form>

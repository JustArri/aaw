<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { MidiInputAutoConnection } from '$lib/generated/models';
    import type { MidiInputsSection } from '$lib/sections/midi-inputs.svelte';

    interface Props {
        section: MidiInputsSection;
        connection: MidiInputAutoConnection;
    }

    let { section, connection }: Props = $props();
</script>

<article class="flex items-center gap-3 px-3.75 py-3 min-[56.25rem]:px-5">
    <span
        class="bg-surface-icon text-foreground-muted grid size-8.5 shrink-0 place-items-center rounded-lg">
        <Icon icon="lucide:refresh-cw" width="0.875rem" />
    </span>
    <div class="min-w-0 flex-1">
        <strong class="block truncate text-sm font-semibold">
            {section.portName(connection.port_id)}
        </strong>
        <small class="text-foreground-muted mt-0.5 block text-xs">
            {connection.preferred_slot_id === null
                ? 'Any available slot'
                : `Slot ${connection.preferred_slot_id + 1}${connection.slot_id_fixed ? ' only' : ' preferred'}`}
        </small>
    </div>
    <Button
        variant="danger-ghost"
        size="sm"
        square
        class="size-8.5"
        type="button"
        aria-label={`Delete auto-connection for ${section.portName(connection.port_id)}`}
        busy={section.deletingPortId === connection.port_id}
        disabled={section.deletingPortId !== null}
        onclick={() => void section.removeAutoConnection(connection.port_id)}>
        <Icon
            icon={section.deletingPortId === connection.port_id
                ? 'lucide:loader-circle'
                : 'lucide:trash-2'}
            width="0.875rem"
            class={section.deletingPortId === connection.port_id ? 'animate-spin' : ''} />
    </Button>
</article>

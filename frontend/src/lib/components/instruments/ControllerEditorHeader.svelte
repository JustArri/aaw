<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import TimedConfirmButton from '$lib/components/TimedConfirmButton.svelte';
    import type { InstrumentsSection } from '$lib/sections/instruments.svelte';

    interface Props {
        section: InstrumentsSection;
    }

    let { section }: Props = $props();
</script>

<header class="border-border flex min-h-15 items-center justify-between gap-3 border-b px-4 py-2.5">
    <div class="min-w-0">
        <strong class="text-foreground block truncate text-base">
            Controller #{section.selectedSlot!.controller_id + 1}
        </strong>
        <small class="text-foreground-muted block truncate text-xs">
            {section.displayName(section.selectedSlot!)}
        </small>
    </div>
    {#if section.embedded}
        <div class="flex flex-wrap items-center justify-end gap-1.5">
            {#if section.entries.length > 1}
                <SelectDropdown
                    compact
                    value={String(section.selectedId)}
                    options={section.entries.map((entry) => ({
                        value: String(entry.id),
                        label: section.displayName(entry.slot),
                        description: `Controller #${entry.slot.controller_id + 1}`
                    }))}
                    onchange={(value) => section.selectInstrument(Number(value))}
                    ariaLabel="Selected MIDI input controller" />
            {/if}
            <Button
                variant={section.selectedSlot!.enabled ? 'accent-outline' : 'secondary'}
                size="sm"
                type="button"
                aria-label={`${section.selectedSlot!.enabled ? 'Disable' : 'Enable'} ${section.displayName(section.selectedSlot!)}`}
                aria-pressed={section.selectedSlot!.enabled}
                busy={section.togglingId === section.selectedId}
                disabled={section.togglingId !== null}
                onclick={() =>
                    void section.toggleEnabled(
                        section.selectedId!,
                        !section.selectedSlot!.enabled
                    )}>
                <Icon
                    icon={section.togglingId === section.selectedId
                        ? 'lucide:loader-circle'
                        : 'lucide:power'}
                    width="0.875rem"
                    class={section.togglingId === section.selectedId ? 'animate-spin' : ''} />
                {section.selectedSlot!.enabled ? 'Enabled' : 'Disabled'}
            </Button>
            {#if section.showAddButton}
                <Button
                    variant="secondary"
                    size="sm"
                    square
                    class="size-8.5"
                    type="button"
                    aria-label="Add MIDI input controller"
                    busy={section.addingInstrument}
                    disabled={section.busy}
                    onclick={() => void section.appendInstrument()}>
                    <Icon
                        icon={section.addingInstrument ? 'lucide:loader-circle' : 'lucide:plus'}
                        width="1.125rem"
                        class={section.addingInstrument ? 'animate-spin' : ''} />
                </Button>
            {/if}
            <TimedConfirmButton
                label="Remove"
                confirmLabel="Confirm remove"
                busyLabel="Removing"
                busy={section.removingInstrument}
                disabled={section.busy}
                onconfirm={() => section.deleteSelected()} />
        </div>
    {/if}
</header>

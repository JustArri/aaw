<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import type { MidiInputsSection } from '$lib/sections/midi-inputs.svelte';
    import InputSlotRow from './InputSlotRow.svelte';

    interface Props {
        section: MidiInputsSection;
    }

    let { section }: Props = $props();
</script>

<section class="border-border bg-surface-card min-w-0 rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                Input slots
            </h2>
            <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                {section.ports.length}
                {section.ports.length === 1 ? 'port' : 'ports'} available
            </p>
        </div>
        <div class="flex items-center gap-2">
            <span
                class="bg-surface-count text-foreground-muted rounded-full px-2.5 py-1 font-mono text-sm">
                {section.slots.filter((slot) => slot.type === 'Connected').length} / {section.slots
                    .length}
            </span>
            <Button
                variant="secondary"
                size="md"
                square
                class="bg-surface-card text-foreground-muted"
                type="button"
                aria-label="Refresh MIDI inputs"
                busy={section.refreshing}
                disabled={section.refreshing}
                onclick={() => void section.refresh()}>
                <Icon
                    icon={section.refreshing
                        ? 'lucide:loader-circle'
                        : section.refreshDone
                          ? 'lucide:check'
                          : 'lucide:refresh-cw'}
                    width="1rem"
                    class={section.refreshing
                        ? 'animate-spin'
                        : section.refreshDone
                          ? 'text-success'
                          : ''} />
            </Button>
        </div>
    </header>

    {#if section.slots.length === 0}
        <div class="grid min-h-40 place-items-center p-5 text-center">
            <div>
                <Icon
                    icon="lucide:unplug"
                    width="1.5rem"
                    class="text-foreground-inactive mx-auto mb-2" />
                <p class="text-foreground-muted m-0 text-sm">No MIDI input slots are configured.</p>
            </div>
        </div>
    {:else}
        <div class="divide-border divide-y">
            {#each section.slots as slot, slotId}
                <InputSlotRow {section} {slot} {slotId} />
            {/each}
        </div>
    {/if}
</section>

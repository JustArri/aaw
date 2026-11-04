<script lang="ts">
    import type { MidiEventsSection } from '$lib/sections/midi-events.svelte';
    import type { MidiEventEntry } from '$lib/state/midi-events.svelte';

    interface Props {
        section: MidiEventsSection;
        entry: MidiEventEntry;
    }

    let { section, entry }: Props = $props();
    const details = $derived(section.describeEvent(entry.event));
</script>

<tr class="hover:bg-surface-hover text-sm min-[56.25rem]:text-sm">
    <td class="text-foreground-muted px-4 py-3 font-mono tabular-nums">
        {section.formatTime(entry.receivedAt)}
    </td>
    <td class="px-3 py-3 text-center">
        <span
            class="bg-surface-badge text-foreground-strong inline-grid size-6 place-items-center rounded-md font-mono font-semibold">
            {entry.event.slot + 1}
        </span>
    </td>
    <td class="px-3 py-3 text-center">
        <span
            class="bg-surface-badge text-foreground-strong inline-grid size-6 place-items-center rounded-md font-mono font-semibold">
            {entry.event.channel + 1}
        </span>
    </td>
    <td class="px-3 py-3 font-semibold">{details.name}</td>
    <td class="px-3 py-3">
        <div class="flex items-center gap-x-5 gap-y-1">
            {#each details.data as datum}
                <span class="whitespace-nowrap">
                    <small class="text-foreground-muted mr-2 text-xs">{datum.name}</small>
                    <span class="font-mono tabular-nums">{datum.value}</span>
                </span>
            {/each}
        </div>
    </td>
</tr>

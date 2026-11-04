<script lang="ts">
    import type { MidiEventsSection } from '$lib/sections/midi-events.svelte';
    import MidiEventRow from './MidiEventRow.svelte';

    interface Props {
        section: MidiEventsSection;
    }

    let { section }: Props = $props();
</script>

<div
    class="w-full max-w-full min-w-0 flex-1 overflow-auto overscroll-contain contain-[inline-size]">
    <table class="w-full min-w-136 border-collapse text-left">
        <thead
            class="bg-surface-inset text-foreground-muted text-xs font-bold tracking-[.12em] uppercase">
            <tr class="border-border border-b">
                <th class="font-inherit w-34 px-4 py-2.5">Time</th>
                <th class="font-inherit w-16 px-3 py-2.5 text-center">Slot</th>
                <th class="font-inherit w-16 px-3 py-2.5 text-center">Ch</th>
                <th class="font-inherit w-44 px-3 py-2.5">Event</th>
                <th class="font-inherit px-3 py-2.5">Data</th>
            </tr>
        </thead>
        <tbody class="divide-border divide-y">
            {#if section.events.length === 0}
                <tr>
                    <td class="text-foreground-muted h-40 px-4 text-center text-sm" colspan="5">
                        Play a note or move a MIDI control to see events here.
                    </td>
                </tr>
            {:else}
                {#each section.events as entry (entry.id)}
                    <MidiEventRow {section} {entry} />
                {/each}
            {/if}
        </tbody>
    </table>
</div>

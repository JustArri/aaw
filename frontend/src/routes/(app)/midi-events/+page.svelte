<script lang="ts">
    import Icon from '@iconify/svelte';
    import EngineStatusIndicator from '$lib/components/EngineStatusIndicator.svelte';
    import type { MidiEvent } from '$lib/generated/models';
    import { notify } from '$lib/push-notifications.svelte';
    import { clearMidiEvents, midiEventLogSize, midiEvents } from '$lib/state/midi-events.svelte';
    import { unreachable } from '$lib/util';

    type EventDetails = {
        name: string;
        data: { name: string; value: string }[];
    };

    const noteNames = ['C', 'C♯', 'D', 'D♯', 'E', 'F', 'F♯', 'G', 'G♯', 'A', 'A♯', 'B'];

    function formatTime(timestamp: number) {
        const date = new Date(timestamp);
        const hours = String(date.getHours()).padStart(2, '0');
        const minutes = String(date.getMinutes()).padStart(2, '0');
        const seconds = String(date.getSeconds()).padStart(2, '0');
        const milliseconds = String(date.getMilliseconds()).padStart(3, '0');
        return `${hours}:${minutes}:${seconds}.${milliseconds}`;
    }

    function formatNote(key: number) {
        const name = noteNames[key % noteNames.length];
        const octave = Math.floor(key / 12) - 1;
        return `${name}${octave} · ${key}`;
    }

    function describeEvent(event: MidiEvent): EventDetails {
        switch (event.type) {
            case 'NoteOn':
                return {
                    name: 'Note on',
                    data: [
                        { name: 'Key', value: formatNote(event.data.key) },
                        { name: 'Velocity', value: String(event.data.velocity) }
                    ]
                };
            case 'NoteOff':
                return {
                    name: 'Note off',
                    data: [
                        { name: 'Key', value: formatNote(event.data.key) },
                        { name: 'Velocity', value: String(event.data.velocity) }
                    ]
                };
            case 'ControlChange':
                return {
                    name: 'Control change',
                    data: [
                        { name: 'Controller', value: `CC ${event.data.controller}` },
                        { name: 'Value', value: String(event.data.value) }
                    ]
                };
            case 'PitchBend':
                return {
                    name: 'Pitch bend',
                    data: [{ name: 'Value', value: String(event.data.value) }]
                };
            case 'ProgramChange':
                return {
                    name: 'Program change',
                    data: [{ name: 'Program', value: String(event.data.program) }]
                };
            case 'ChannelAftertouch':
                return {
                    name: 'Channel aftertouch',
                    data: [{ name: 'Pressure', value: String(event.data.value) }]
                };
            case 'PolyphonicAftertouch':
                return {
                    name: 'Poly aftertouch',
                    data: [
                        { name: 'Key', value: formatNote(event.data.key) },
                        { name: 'Pressure', value: String(event.data.value) }
                    ]
                };
            default:
                return unreachable(event);
        }
    }

    function clearEvents() {
        clearMidiEvents();
        notify.success('MIDI event log cleared.', { title: 'Clear events' });
    }
</script>

<section
    class="border-border bg-surface-card flex max-h-[calc(100dvh-9rem)] w-full max-w-full min-w-0 flex-col overflow-hidden rounded-xl border contain-[inline-size] min-[56.25rem]:max-h-[calc(100dvh-8rem)]">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div>
            <div class="flex items-center gap-2">
                <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                    MIDI events
                </h2>
                <EngineStatusIndicator />
            </div>
            <p class="text-foreground-muted mt-1 mb-0 text-sm min-[56.25rem]:text-sm">
                Live input monitor · newest events first
            </p>
        </div>
        <div class="flex items-center gap-2">
            <span
                class="bg-surface-badge text-foreground-strong min-w-7 rounded-md px-2 py-1 text-center font-mono text-sm tabular-nums">
                {midiEvents.length} / {midiEventLogSize()}
            </span>
            <button
                class="border-border-strong bg-surface-card text-foreground-muted hover:bg-surface-hover hover:text-foreground grid size-9 cursor-pointer place-items-center rounded-lg border disabled:cursor-not-allowed disabled:opacity-40"
                type="button"
                aria-label="Clear MIDI events"
                disabled={midiEvents.length === 0}
                onclick={clearEvents}>
                <Icon icon="lucide:trash-2" width="1rem" />
            </button>
        </div>
    </header>

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
                {#if midiEvents.length === 0}
                    <tr>
                        <td class="text-foreground-muted h-40 px-4 text-center text-sm" colspan="5">
                            Play a note or move a MIDI control to see events here.
                        </td>
                    </tr>
                {:else}
                    {#each midiEvents as entry (entry.id)}
                        {@const details = describeEvent(entry.event)}
                        <tr class="hover:bg-surface-hover text-sm min-[56.25rem]:text-sm">
                            <td class="text-foreground-muted px-4 py-3 font-mono tabular-nums">
                                {formatTime(entry.receivedAt)}
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
                                            <small class="text-foreground-muted mr-2 text-xs">
                                                {datum.name}
                                            </small>
                                            <span class="font-mono tabular-nums">
                                                {datum.value}
                                            </span>
                                        </span>
                                    {/each}
                                </div>
                            </td>
                        </tr>
                    {/each}
                {/if}
            </tbody>
        </table>
    </div>
</section>

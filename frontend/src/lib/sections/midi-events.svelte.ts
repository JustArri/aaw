import type { MidiEvent } from '$lib/generated/models';
import { midiNoteNames } from '$lib/content/midi';
import { clearMidiEvents, midiEventLogSize, midiEvents } from '$lib/state/midi-events.svelte';
import { unreachable } from '$lib/util';

type EventDetails = {
    name: string;
    data: { name: string; value: string }[];
};

function formatTime(timestamp: number) {
    const date = new Date(timestamp);
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    const seconds = String(date.getSeconds()).padStart(2, '0');
    const milliseconds = String(date.getMilliseconds()).padStart(3, '0');
    return `${hours}:${minutes}:${seconds}.${milliseconds}`;
}

function formatNote(key: number) {
    const name = midiNoteNames[key % midiNoteNames.length];
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

export class MidiEventsSection {
    get events() {
        return midiEvents;
    }
    get logSize() {
        return midiEventLogSize();
    }
    formatTime(timestamp: number) {
        return formatTime(timestamp);
    }
    describeEvent(event: MidiEvent) {
        return describeEvent(event);
    }
    clearEvents() {
        clearMidiEvents();
    }
}

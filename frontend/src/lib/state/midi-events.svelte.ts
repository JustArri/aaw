import type { MidiEvent } from '$lib/generated/models';
import { onMount } from 'svelte';

const MIDI_EVENT_LOG_SIZE_STORAGE_KEY = 'aaw-midi-event-log-size';

export const DEFAULT_MIDI_EVENT_LOG_SIZE = 256;
export const MIN_MIDI_EVENT_LOG_SIZE = 16;
export const MAX_MIDI_EVENT_LOG_SIZE = 4096;

export type MidiEventEntry = {
    id: number;
    receivedAt: number;
    event: MidiEvent;
};

export const midiEvents = $state<MidiEventEntry[]>([]);

let nextMidiEventId = 1;
let midiEventLogSizeState = $state(DEFAULT_MIDI_EVENT_LOG_SIZE);
const midiEventListeners = new Set<(entry: MidiEventEntry) => void>();

export function midiEventLogSize() {
    return midiEventLogSizeState;
}

export function setMidiEventLogSize(value: number) {
    if (!Number.isFinite(value)) {
        return;
    }

    midiEventLogSizeState = Math.max(
        MIN_MIDI_EVENT_LOG_SIZE,
        Math.min(MAX_MIDI_EVENT_LOG_SIZE, Math.round(value))
    );
    localStorage.setItem(MIDI_EVENT_LOG_SIZE_STORAGE_KEY, String(midiEventLogSizeState));

    if (midiEvents.length > midiEventLogSizeState) {
        midiEvents.length = midiEventLogSizeState;
    }
}

export function autoloadMidiEventLogSize() {
    onMount(() => {
        const storedValue = localStorage.getItem(MIDI_EVENT_LOG_SIZE_STORAGE_KEY);

        if (storedValue === null) {
            return;
        }

        setMidiEventLogSize(Number(storedValue));
    });
}

export function recordMidiEvent(event: MidiEvent) {
    const entry = {
        id: nextMidiEventId++,
        receivedAt: Date.now(),
        event
    };
    midiEvents.unshift(entry);

    for (const listener of midiEventListeners) {
        listener(entry);
    }

    if (midiEvents.length > midiEventLogSizeState) {
        midiEvents.length = midiEventLogSizeState;
    }
}

export function subscribeMidiEvents(listener: (entry: MidiEventEntry) => void) {
    midiEventListeners.add(listener);
    return () => {
        midiEventListeners.delete(listener);
    };
}

export function clearMidiEvents() {
    midiEvents.splice(0, midiEvents.length);
}

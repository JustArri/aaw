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

export type ActiveMidiNote = {
    key: number;
    velocity: number;
    slot: number;
    channel: number;
    startedAt: number;
};

export type PlayedMidiNote = ActiveMidiNote & {
    sequence: number;
};

export type MidiKeyActivity = {
    key: number;
    channel: number;
    value: number;
    updatedAt: number;
};

export const MIDI_KEY_ACTIVITY_DECAY_MS = 3000;
export const MIDI_KEY_ACTIVITY_THRESHOLD = 0.08;
const MAX_MIDI_KEY_ACTIVITY = 4;

export const midiEvents = $state<MidiEventEntry[]>([]);

let nextMidiEventId = 1;
let nextPlayedNoteSequence = 1;
let midiEventLogSizeState = $state(DEFAULT_MIDI_EVENT_LOG_SIZE);
let activeMidiNotesState = $state({} as Record<string, ActiveMidiNote>);
let latestPlayedMidiNoteState = $state(null as PlayedMidiNote | null);
let midiKeyActivitiesState = $state({} as Record<string, MidiKeyActivity>);
const midiEventListeners = new Set<(entry: MidiEventEntry) => void>();

export function midiEventLogSize() {
    return midiEventLogSizeState;
}

export function activeMidiNotes() {
    return Object.values(activeMidiNotesState);
}

export function latestPlayedMidiNote() {
    return latestPlayedMidiNoteState;
}

export function midiKeyActivities() {
    return Object.values(midiKeyActivitiesState);
}

export function midiKeyActivityAt(activity: MidiKeyActivity, timestamp: number) {
    return (
        activity.value * Math.exp(-(timestamp - activity.updatedAt) / MIDI_KEY_ACTIVITY_DECAY_MS)
    );
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
    updateActiveMidiNotes(event);

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

function midiNoteId(slot: number, channel: number, key: number) {
    return `${slot}:${channel}:${key}`;
}

function updateActiveMidiNotes(event: MidiEvent) {
    switch (event.type) {
        case 'NoteOn': {
            const note = {
                key: event.data.key,
                velocity: event.data.velocity,
                slot: event.slot,
                channel: event.channel,
                startedAt: Date.now()
            };
            activeMidiNotesState = {
                ...activeMidiNotesState,
                [midiNoteId(event.slot, event.channel, event.data.key)]: note
            };
            latestPlayedMidiNoteState = {
                ...note,
                sequence: nextPlayedNoteSequence++
            };
            const activityId = `${note.channel}:${note.key}`;
            const previousActivity = midiKeyActivitiesState[activityId];
            const previousValue = previousActivity
                ? midiKeyActivityAt(previousActivity, note.startedAt)
                : 0;
            midiKeyActivitiesState = {
                ...midiKeyActivitiesState,
                [activityId]: {
                    key: note.key,
                    channel: note.channel,
                    value: Math.min(MAX_MIDI_KEY_ACTIVITY, previousValue + 1),
                    updatedAt: note.startedAt
                }
            };
            break;
        }
        case 'NoteOff': {
            const id = midiNoteId(event.slot, event.channel, event.data.key);
            if (activeMidiNotesState[id]) {
                const next = { ...activeMidiNotesState };
                delete next[id];
                activeMidiNotesState = next;
            }
            break;
        }
        case 'ControlChange':
            if (event.data.controller === 120 || event.data.controller === 123) {
                clearActiveMidiNotes(event.slot, event.channel);
            }
            break;
    }
}

export function clearActiveMidiNotes(slot?: number, channel?: number) {
    if (slot === undefined || channel === undefined) {
        activeMidiNotesState = {};
        latestPlayedMidiNoteState = null;
        return;
    }

    activeMidiNotesState = Object.fromEntries(
        Object.entries(activeMidiNotesState).filter(
            ([, note]) => note.slot !== slot || note.channel !== channel
        )
    );
}

export function clearMidiKeyActivities() {
    midiKeyActivitiesState = {};
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

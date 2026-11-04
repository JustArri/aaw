import {
    MAX_MIDI_EVENT_LOG_SIZE,
    midiEventLogSize,
    MIN_MIDI_EVENT_LOG_SIZE,
    setMidiEventLogSize
} from '$lib/state/midi-events.svelte';
import { accentPresets, themeModes } from '$lib/content/appearance';
import {
    accent,
    accentHex,
    applyTheme,
    setAccent,
    setAccentHex,
    themeMode,
    type Accent,
    type ThemeMode
} from '$lib/state/theme.svelte';

export class SettingsSection {
    readonly accentPresets = accentPresets;
    readonly themeModes = themeModes;
    readonly minMidiEventLogSize = MIN_MIDI_EVENT_LOG_SIZE;
    readonly maxMidiEventLogSize = MAX_MIDI_EVENT_LOG_SIZE;

    get accent() {
        return accent();
    }
    get accentHex() {
        return accentHex();
    }
    get themeMode() {
        return themeMode();
    }
    get midiEventLogSize() {
        return midiEventLogSize();
    }
    selectTheme(mode: ThemeMode) {
        applyTheme(mode);
    }
    selectAccent(value: Accent) {
        setAccent(value);
    }
    setAccentHex(value: string) {
        setAccentHex(value);
    }
    updateMidiEventLogSize(value: number) {
        setMidiEventLogSize(value);
    }
}

export type { Accent };

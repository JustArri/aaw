import { onMount } from 'svelte';
import {
    blackKeyWidthRatio,
    buildKeyboard,
    keyboardActivityTimerIntervalMs,
    keyboardAutoViewPaddingKeys,
    keyboardAutoViewStorageKey,
    whiteKeyWidthRem
} from '$lib/content/keyboard';
import { midiChannelColors, midiNoteNames } from '$lib/content/midi';
import {
    activeMidiNotes,
    clearActiveMidiNotes,
    clearMidiKeyActivities,
    latestPlayedMidiNote,
    midiKeyActivities,
    midiKeyActivityAt,
    MIDI_KEY_ACTIVITY_THRESHOLD,
    type ActiveMidiNote
} from '$lib/state/midi-events.svelte';

export class KeyboardSection {
    readonly whiteKeyWidthRem = whiteKeyWidthRem;
    readonly blackKeyWidthRatio = blackKeyWidthRatio;
    readonly keyboard = buildKeyboard();
    readonly whiteKeys = this.keyboard.keys.filter((key) => !key.black);
    readonly blackKeys = this.keyboard.keys.filter((key) => key.black);
    readonly channelColors = midiChannelColors;

    keyboardViewport = $state<HTMLDivElement>();
    initialPositionSet = $state(false);
    viewportWidth = $state(0);
    keyWidth = $state(whiteKeyWidthRem);
    autoView = $state(true);
    channelHelpOpen = $state(false);
    enabledChannels = $state(Array.from({ length: 16 }, () => true));
    activityTime = $state(Date.now());

    allNotes = $derived(activeMidiNotes());
    notes = $derived(this.allNotes.filter((note) => this.enabledChannels[note.channel] ?? false));
    latestNote = $derived(latestPlayedMidiNote());
    keyActivities = $derived(midiKeyActivities());
    enabledChannelCount = $derived(this.enabledChannels.filter(Boolean).length);
    hasRecentActivity = $derived(
        this.keyActivities.some(
            (activity) =>
                midiKeyActivityAt(activity, this.activityTime) >= MIDI_KEY_ACTIVITY_THRESHOLD
        )
    );
    playedRangeCode = $derived.by(() => {
        let min: number | null = null;
        let max: number | null = null;
        for (const activity of this.keyActivities) {
            if (
                !this.enabledChannels[activity.channel] ||
                midiKeyActivityAt(activity, this.activityTime) < MIDI_KEY_ACTIVITY_THRESHOLD
            )
                continue;
            min = Math.min(min ?? activity.key, activity.key);
            max = Math.max(max ?? activity.key, activity.key);
        }
        for (const note of this.notes) {
            min = Math.min(min ?? note.key, note.key);
            max = Math.max(max ?? note.key, note.key);
        }
        return min === null || max === null ? null : min * 128 + max;
    });
    playedRange = $derived(
        this.playedRangeCode === null
            ? null
            : {
                  min: Math.floor(this.playedRangeCode / 128),
                  max: this.playedRangeCode % 128
              }
    );
    notesByKey = $derived.by(() => {
        const grouped = new Map<number, ActiveMidiNote[]>();
        for (const note of this.notes) {
            const keyNotes = grouped.get(note.key);
            if (keyNotes) keyNotes.push(note);
            else grouped.set(note.key, [note]);
        }
        return grouped;
    });
    activeKeys = $derived(
        [...this.notesByKey.entries()]
            .map(([key, keyNotes]) => ({
                key,
                notes: keyNotes,
                velocity: Math.max(...keyNotes.map((note) => note.velocity))
            }))
            .sort((left, right) => left.key - right.key)
    );

    constructor() {
        onMount(() => {
            this.autoView = localStorage.getItem(keyboardAutoViewStorageKey) !== 'false';
            const resizeObserver = new ResizeObserver(([entry]) => {
                this.viewportWidth = Math.max(this.viewportWidth, entry.contentRect.width);
            });
            if (this.keyboardViewport) resizeObserver.observe(this.keyboardViewport);
            const activityTimer = window.setInterval(() => {
                this.activityTime = Date.now();
            }, keyboardActivityTimerIntervalMs);
            requestAnimationFrame(() => {
                this.centerKey(this.latestNote?.key ?? 60, 'auto');
                this.initialPositionSet = true;
            });
            return () => {
                resizeObserver.disconnect();
                window.clearInterval(activityTimer);
            };
        });

        $effect(() => {
            const min = this.playedRange?.min;
            const max = this.playedRange?.max;
            const width = this.viewportWidth;
            if (
                !this.initialPositionSet ||
                !this.autoView ||
                min === undefined ||
                max === undefined ||
                width === 0
            )
                return;
            this.updateAutoView(min, max);
        });
    }

    noteName(key: number) {
        return `${midiNoteNames[key % 12]}${Math.floor(key / 12) - 1}`;
    }

    channelColor(channel: number) {
        return this.channelColors[channel % this.channelColors.length];
    }

    noteColor(keyNotes: ActiveMidiNote[]) {
        const colors = [...new Set(keyNotes.map((note) => this.channelColor(note.channel)))];
        if (colors.length === 1) return colors[0];
        const stops = colors.flatMap((color, index) => {
            const start = (index / colors.length) * 100;
            const end = ((index + 1) / colors.length) * 100;
            return [`${color} ${start}%`, `${color} ${end}%`];
        });
        return `linear-gradient(to right, ${stops.join(', ')})`;
    }

    private keyboardKey(note: number) {
        return this.keyboard.keys[note];
    }

    private keyBounds(note: number) {
        const key = this.keyboardKey(note);
        if (key.black) {
            const left = key.left / whiteKeyWidthRem;
            return { left, right: left + blackKeyWidthRatio };
        }
        return { left: key.whiteIndex, right: key.whiteIndex + 1 };
    }

    centerKey(key: number, behavior: ScrollBehavior) {
        const element = this.keyboardViewport?.querySelector<HTMLElement>(
            `[data-midi-key="${key}"]`
        );
        if (!element || !this.keyboardViewport) return;
        const left =
            element.offsetLeft - (this.keyboardViewport.clientWidth - element.offsetWidth) / 2;
        this.keyboardViewport.scrollTo({ left: Math.max(0, left), behavior });
    }

    private centerRange(min: number, max: number, behavior: ScrollBehavior) {
        const minElement = this.keyboardViewport?.querySelector<HTMLElement>(
            `[data-midi-key="${min}"]`
        );
        const maxElement = this.keyboardViewport?.querySelector<HTMLElement>(
            `[data-midi-key="${max}"]`
        );
        if (!minElement || !maxElement || !this.keyboardViewport) return;
        const left = Math.min(minElement.offsetLeft, maxElement.offsetLeft);
        const right = Math.max(
            minElement.offsetLeft + minElement.offsetWidth,
            maxElement.offsetLeft + maxElement.offsetWidth
        );
        const target = (left + right - this.keyboardViewport.clientWidth) / 2;
        this.keyboardViewport.scrollTo({ left: Math.max(0, target), behavior });
    }

    private updateAutoView(min: number, max: number) {
        if (!this.keyboardViewport || this.viewportWidth === 0) return;
        const rootFontSize = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
        const minBounds = this.keyBounds(min);
        const maxBounds = this.keyBounds(max);
        const rangeWidth =
            Math.max(minBounds.right, maxBounds.right) -
            Math.min(minBounds.left, maxBounds.left) +
            keyboardAutoViewPaddingKeys * 2;
        const availableWidth = Math.max(1, this.viewportWidth - rootFontSize * 2);
        this.keyWidth = Math.min(
            whiteKeyWidthRem,
            Math.max(0.2, availableWidth / rootFontSize / rangeWidth)
        );
        requestAnimationFrame(() => this.centerRange(min, max, 'smooth'));
    }

    setAutoView(enabled: boolean) {
        this.autoView = enabled;
        localStorage.setItem(keyboardAutoViewStorageKey, String(enabled));
        if (!enabled) this.keyWidth = whiteKeyWidthRem;
    }

    setChannelEnabled(channel: number, enabled: boolean) {
        this.enabledChannels = this.enabledChannels.map((current, index) =>
            index === channel ? enabled : current
        );
    }

    setAllChannelsEnabled(enabled: boolean) {
        this.enabledChannels = this.enabledChannels.map(() => enabled);
    }

    clearKeyboard() {
        clearActiveMidiNotes();
        clearMidiKeyActivities();
        this.keyWidth = whiteKeyWidthRem;
        requestAnimationFrame(() => this.centerKey(60, 'smooth'));
    }
}

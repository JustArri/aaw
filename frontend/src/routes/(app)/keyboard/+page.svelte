<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import EngineStatusIndicator from '$lib/components/EngineStatusIndicator.svelte';
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

    const NOTE_NAMES = ['C', 'C♯', 'D', 'D♯', 'E', 'F', 'F♯', 'G', 'G♯', 'A', 'A♯', 'B'];
    const BLACK_KEY_CLASSES = new Set([1, 3, 6, 8, 10]);
    const AUTO_VIEW_STORAGE_KEY = 'aaw-keyboard-auto-view';
    const WHITE_KEY_WIDTH_REM = 2;
    const BLACK_KEY_WIDTH_REM = 1.25;
    const BLACK_KEY_WIDTH_RATIO = BLACK_KEY_WIDTH_REM / WHITE_KEY_WIDTH_REM;
    const AUTO_VIEW_PADDING_KEYS = 2;
    const ACTIVITY_TIMER_INTERVAL_MS = 250;
    const CHANNEL_COLORS = Array.from(
        { length: 16 },
        (_, index) => `var(--midi-channel-${index + 1})`
    );

    type KeyboardKey = {
        note: number;
        whiteIndex: number;
        black: boolean;
        left: number;
    };

    function buildKeyboard() {
        const keys: KeyboardKey[] = [];
        let whiteCount = 0;

        for (let note = 0; note < 128; note += 1) {
            const black = BLACK_KEY_CLASSES.has(note % 12);
            if (black) {
                keys.push({
                    note,
                    whiteIndex: -1,
                    black,
                    left: whiteCount * WHITE_KEY_WIDTH_REM - BLACK_KEY_WIDTH_REM / 2
                });
            } else {
                keys.push({
                    note,
                    whiteIndex: whiteCount,
                    black,
                    left: whiteCount * WHITE_KEY_WIDTH_REM
                });
                whiteCount += 1;
            }
        }

        return { keys, whiteCount };
    }

    const keyboard = buildKeyboard();
    const whiteKeys = keyboard.keys.filter((key) => !key.black);
    const blackKeys = keyboard.keys.filter((key) => key.black);

    let keyboardViewport: HTMLDivElement;
    let initialPositionSet = $state(false);
    let viewportWidth = $state(0);
    let keyWidth = $state(WHITE_KEY_WIDTH_REM);
    let autoView = $state(true);
    let channelHelpOpen = $state(false);
    let enabledChannels = $state(Array.from({ length: 16 }, () => true));
    let activityTime = $state(Date.now());

    let allNotes = $derived(activeMidiNotes());
    let notes = $derived(allNotes.filter((note) => enabledChannels[note.channel] ?? false));
    let latestNote = $derived(latestPlayedMidiNote());
    let keyActivities = $derived(midiKeyActivities());
    let enabledChannelCount = $derived(enabledChannels.filter(Boolean).length);
    let hasRecentActivity = $derived(
        keyActivities.some(
            (activity) => midiKeyActivityAt(activity, activityTime) >= MIDI_KEY_ACTIVITY_THRESHOLD
        )
    );
    let playedRangeCode = $derived.by(() => {
        let min: number | null = null;
        let max: number | null = null;

        for (const activity of keyActivities) {
            if (
                !enabledChannels[activity.channel] ||
                midiKeyActivityAt(activity, activityTime) < MIDI_KEY_ACTIVITY_THRESHOLD
            ) {
                continue;
            }

            min = Math.min(min ?? activity.key, activity.key);
            max = Math.max(max ?? activity.key, activity.key);
        }

        for (const note of notes) {
            min = Math.min(min ?? note.key, note.key);
            max = Math.max(max ?? note.key, note.key);
        }

        return min === null || max === null ? null : min * 128 + max;
    });
    let playedRange = $derived(
        playedRangeCode === null
            ? null
            : { min: Math.floor(playedRangeCode / 128), max: playedRangeCode % 128 }
    );
    let notesByKey = $derived.by(() => {
        const grouped = new Map<number, ActiveMidiNote[]>();

        for (const note of notes) {
            const keyNotes = grouped.get(note.key);
            if (keyNotes) {
                keyNotes.push(note);
            } else {
                grouped.set(note.key, [note]);
            }
        }

        return grouped;
    });

    let activeKeys = $derived(
        [...notesByKey.entries()]
            .map(([key, keyNotes]) => ({
                key,
                notes: keyNotes,
                velocity: Math.max(...keyNotes.map((note) => note.velocity))
            }))
            .sort((left, right) => left.key - right.key)
    );

    function noteName(key: number) {
        return `${NOTE_NAMES[key % 12]}${Math.floor(key / 12) - 1}`;
    }

    function channelColor(channel: number) {
        return CHANNEL_COLORS[channel % CHANNEL_COLORS.length];
    }

    function noteColor(keyNotes: ActiveMidiNote[]) {
        const colors = [...new Set(keyNotes.map((note) => channelColor(note.channel)))];
        if (colors.length === 1) {
            return colors[0];
        }

        const stops = colors.flatMap((color, index) => {
            const start = (index / colors.length) * 100;
            const end = ((index + 1) / colors.length) * 100;
            return [`${color} ${start}%`, `${color} ${end}%`];
        });
        return `linear-gradient(to right, ${stops.join(', ')})`;
    }

    function keyboardKey(note: number) {
        return keyboard.keys[note];
    }

    function keyBounds(note: number) {
        const key = keyboardKey(note);
        if (key.black) {
            const left = key.left / WHITE_KEY_WIDTH_REM;
            return {
                left,
                right: left + BLACK_KEY_WIDTH_RATIO
            };
        }

        return { left: key.whiteIndex, right: key.whiteIndex + 1 };
    }

    function centerKey(key: number, behavior: ScrollBehavior) {
        const element = keyboardViewport?.querySelector<HTMLElement>(`[data-midi-key="${key}"]`);
        if (!element) {
            return;
        }

        const left = element.offsetLeft - (keyboardViewport.clientWidth - element.offsetWidth) / 2;
        keyboardViewport.scrollTo({ left: Math.max(0, left), behavior });
    }

    function centerRange(min: number, max: number, behavior: ScrollBehavior) {
        const minElement = keyboardViewport?.querySelector<HTMLElement>(`[data-midi-key="${min}"]`);
        const maxElement = keyboardViewport?.querySelector<HTMLElement>(`[data-midi-key="${max}"]`);
        if (!minElement || !maxElement) {
            return;
        }

        const left = Math.min(minElement.offsetLeft, maxElement.offsetLeft);
        const right = Math.max(
            minElement.offsetLeft + minElement.offsetWidth,
            maxElement.offsetLeft + maxElement.offsetWidth
        );
        const target = (left + right - keyboardViewport.clientWidth) / 2;
        keyboardViewport.scrollTo({ left: Math.max(0, target), behavior });
    }

    function updateAutoView(min: number, max: number) {
        if (!keyboardViewport || viewportWidth === 0) {
            return;
        }

        const rootFontSize = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
        const minBounds = keyBounds(min);
        const maxBounds = keyBounds(max);
        const rangeWidth =
            Math.max(minBounds.right, maxBounds.right) -
            Math.min(minBounds.left, maxBounds.left) +
            AUTO_VIEW_PADDING_KEYS * 2;
        const availableWidth = Math.max(1, viewportWidth - rootFontSize * 2);

        keyWidth = Math.min(
            WHITE_KEY_WIDTH_REM,
            Math.max(0.2, availableWidth / rootFontSize / rangeWidth)
        );

        requestAnimationFrame(() => centerRange(min, max, 'smooth'));
    }

    function setAutoView(enabled: boolean) {
        autoView = enabled;
        localStorage.setItem(AUTO_VIEW_STORAGE_KEY, String(enabled));

        if (!enabled) {
            keyWidth = WHITE_KEY_WIDTH_REM;
        }
    }

    function setChannelEnabled(channel: number, enabled: boolean) {
        enabledChannels = enabledChannels.map((current, index) =>
            index === channel ? enabled : current
        );
    }

    function setAllChannelsEnabled(enabled: boolean) {
        enabledChannels = enabledChannels.map(() => enabled);
    }

    function clearKeyboard() {
        clearActiveMidiNotes();
        clearMidiKeyActivities();
        keyWidth = WHITE_KEY_WIDTH_REM;
        requestAnimationFrame(() => centerKey(60, 'smooth'));
    }

    onMount(() => {
        autoView = localStorage.getItem(AUTO_VIEW_STORAGE_KEY) !== 'false';
        const resizeObserver = new ResizeObserver(([entry]) => {
            viewportWidth = Math.max(viewportWidth, entry.contentRect.width);
        });
        resizeObserver.observe(keyboardViewport);
        const activityTimer = window.setInterval(() => {
            activityTime = Date.now();
        }, ACTIVITY_TIMER_INTERVAL_MS);

        requestAnimationFrame(() => {
            centerKey(latestNote?.key ?? 60, 'auto');
            initialPositionSet = true;
        });

        return () => {
            resizeObserver.disconnect();
            window.clearInterval(activityTimer);
        };
    });

    $effect(() => {
        const min = playedRange?.min;
        const max = playedRange?.max;
        const width = viewportWidth;

        if (
            !initialPositionSet ||
            !autoView ||
            min === undefined ||
            max === undefined ||
            width === 0
        ) {
            return;
        }

        updateAutoView(min, max);
    });
</script>

<section class="border-border bg-surface-card overflow-hidden rounded-xl border">
    <header
        class="border-border flex min-h-16.75 items-center justify-between gap-4 border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5">
        <div class="min-w-0">
            <div class="flex items-center gap-2">
                <h2 class="text-foreground m-0 text-base font-semibold min-[56.25rem]:text-lg">
                    Keyboard
                </h2>
                <EngineStatusIndicator />
            </div>
            <p class="text-foreground-muted mt-1 mb-0 flex items-center gap-1.5 truncate text-sm">
                {#if latestNote}
                    <span
                        class="size-2 shrink-0 rounded-full"
                        style:background-color={channelColor(latestNote.channel)}>
                    </span>
                    Input {latestNote.slot + 1} · Ch {latestNote.channel + 1}
                {:else}
                    Waiting for MIDI input
                {/if}
            </p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
            <div class="group relative">
                <button
                    class={`grid size-9 cursor-pointer place-items-center rounded-lg border transition-colors ${enabledChannelCount === CHANNEL_COLORS.length ? 'border-border-strong bg-surface-card text-foreground-muted hover:bg-surface-hover hover:text-foreground' : 'border-accent/40 bg-accent/10 text-accent hover:bg-accent/15'}`}
                    type="button"
                    aria-label="Filter MIDI channels"
                    aria-expanded={channelHelpOpen}
                    aria-controls="midi-channel-color-help"
                    onclick={() => {
                        channelHelpOpen = !channelHelpOpen;
                    }}
                    onblur={() => {
                        channelHelpOpen = false;
                    }}>
                    <Icon icon="lucide:info" width="1rem" />
                </button>
                <div
                    id="midi-channel-color-help"
                    class={`border-border bg-surface-overlay absolute top-full right-0 z-30 mt-2 w-56 rounded-xl border p-3 shadow-[0_0.75rem_2rem_var(--overlay-shadow)] transition-[visibility,opacity,transform] ${channelHelpOpen ? 'visible translate-y-0 opacity-100' : 'invisible -translate-y-1 opacity-0 group-focus-within:visible group-focus-within:translate-y-0 group-focus-within:opacity-100 group-hover:visible group-hover:translate-y-0 group-hover:opacity-100'}`}
                    role="group"
                    aria-label="MIDI channel filters">
                    <div class="mb-2 flex items-center justify-between gap-2">
                        <strong class="text-foreground block text-sm font-semibold">
                            MIDI channels
                        </strong>
                        <div class="border-border-strong flex overflow-hidden rounded-md border">
                            <button
                                class="text-foreground-muted hover:bg-surface-hover hover:text-foreground cursor-pointer border-0 bg-transparent px-2 py-1 text-xs font-semibold"
                                type="button"
                                onclick={() => setAllChannelsEnabled(true)}>
                                All
                            </button>
                            <button
                                class="border-border-strong text-foreground-muted hover:bg-surface-hover hover:text-foreground cursor-pointer border-0 border-l bg-transparent px-2 py-1 text-xs font-semibold"
                                type="button"
                                onclick={() => setAllChannelsEnabled(false)}>
                                None
                            </button>
                        </div>
                    </div>
                    <div class="grid grid-cols-2 gap-x-3 gap-y-2">
                        {#each CHANNEL_COLORS as color, channel}
                            <button
                                class={`flex min-h-7 cursor-pointer items-center gap-2 rounded-md border px-1.5 text-left text-[0.6875rem] font-medium whitespace-nowrap transition-colors ${enabledChannels[channel] ? 'text-foreground' : 'border-border text-foreground-muted bg-surface-inset opacity-65 hover:opacity-100'}`}
                                style:border-color={enabledChannels[channel] ? color : undefined}
                                style:background-color={enabledChannels[channel]
                                    ? `color-mix(in srgb, ${color} 12%, transparent)`
                                    : undefined}
                                type="button"
                                aria-pressed={enabledChannels[channel]}
                                onclick={() =>
                                    setChannelEnabled(channel, !enabledChannels[channel])}>
                                <span
                                    class="size-2.5 shrink-0 rounded-full"
                                    style:background-color={enabledChannels[channel]
                                        ? color
                                        : `color-mix(in srgb, ${color} 25%, transparent)`}>
                                </span>
                                Channel {channel + 1}
                            </button>
                        {/each}
                    </div>
                </div>
            </div>
            <span
                class="bg-surface-badge text-foreground-strong min-w-7 rounded-md px-2 py-1 text-center font-mono text-sm tabular-nums"
                title={`${notes.length} active MIDI ${notes.length === 1 ? 'note' : 'notes'}`}>
                {activeKeys.length}
            </span>
        </div>
    </header>

    <div
        class="border-border bg-surface-inset flex items-center gap-2 border-b px-3.75 py-2.5 min-[56.25rem]:px-5">
        <div
            class="border-border-strong bg-surface-card flex h-9 shrink-0 items-stretch overflow-hidden rounded-lg border">
            <span
                class="text-foreground-muted grid min-w-17 place-items-center px-2 font-mono text-xs font-semibold tabular-nums"
                title={playedRange
                    ? `Recent range ${noteName(playedRange.min)} – ${noteName(playedRange.max)}`
                    : 'No recent range'}>
                {playedRange ? `${noteName(playedRange.min)}–${noteName(playedRange.max)}` : '—'}
            </span>
            <button
                class={`border-border-strong grid w-9 cursor-pointer place-items-center border-0 border-l transition-colors ${autoView ? 'bg-accent/12 text-accent hover:bg-accent/18' : 'text-foreground-muted hover:bg-surface-hover hover:text-foreground'}`}
                type="button"
                aria-label={autoView ? 'Disable Auto view' : 'Enable Auto view'}
                aria-pressed={autoView}
                title={`Auto view ${autoView ? 'on' : 'off'}`}
                onclick={() => setAutoView(!autoView)}>
                <Icon icon="lucide:scan" width="1rem" />
            </button>
            <button
                class="border-border-strong text-foreground-muted hover:bg-surface-hover hover:text-foreground grid w-9 cursor-pointer place-items-center border-0 border-l transition-colors disabled:cursor-not-allowed disabled:opacity-40"
                type="button"
                aria-label="Clear held notes and learned range"
                title="Clear held notes and learned range"
                disabled={allNotes.length === 0 && !hasRecentActivity}
                onclick={clearKeyboard}>
                <Icon icon="lucide:eraser" width="1rem" />
            </button>
        </div>
        <div
            class="h-10 min-w-0 flex-1 overflow-x-auto overflow-y-hidden overscroll-x-contain"
            aria-live="polite">
            {#if activeKeys.length === 0}
                <p class="text-foreground-muted m-0 flex h-full items-center text-sm">
                    Play a note to see it here.
                </p>
            {:else}
                <div class="flex h-full w-max items-center gap-2" aria-label="Active notes">
                    {#each activeKeys as activeKey (activeKey.key)}
                        <span
                            class="border-border-strong bg-surface-card text-foreground inline-flex shrink-0 items-center gap-2 rounded-lg border px-2 py-1 text-sm font-semibold"
                            title={activeKey.notes
                                .map(
                                    (note) =>
                                        `Input ${note.slot + 1}, channel ${note.channel + 1}, velocity ${note.velocity}`
                                )
                                .join('\n')}>
                            <span>{noteName(activeKey.key)}</span>
                            <span class="font-mono text-xs tabular-nums">
                                {activeKey.velocity}
                            </span>
                            <span class="flex items-center gap-1" aria-label="MIDI channels">
                                {#each [...new Set(activeKey.notes.map((note) => note.channel))] as channel}
                                    <span
                                        class="size-2 rounded-full"
                                        style:background-color={channelColor(channel)}
                                        title={`Channel ${channel + 1}`}>
                                    </span>
                                {/each}
                            </span>
                        </span>
                    {/each}
                </div>
            {/if}
        </div>
    </div>

    <div
        class="bg-surface-inset-deep w-full touch-pan-x overflow-x-auto overscroll-x-contain p-3.75 min-[56.25rem]:p-5"
        bind:this={keyboardViewport}>
        <div
            class="relative mx-auto h-40 select-none min-[56.25rem]:h-56"
            style:width={`${keyboard.whiteCount * keyWidth}rem`}>
            <div class="absolute inset-0 flex">
                {#each whiteKeys as key (key.note)}
                    {@const active = notesByKey.has(key.note)}
                    {@const keyNotes = notesByKey.get(key.note) ?? []}
                    <div
                        class={`relative h-full shrink-0 border-r border-b first:border-l ${active ? 'border-keyboard-black/35 shadow-[inset_0_-0.3rem_0_rgb(0_0_0/0.22)]' : 'border-keyboard-black/35 bg-keyboard-white'}`}
                        style:width={`${keyWidth}rem`}
                        style:background={active ? noteColor(keyNotes) : undefined}
                        data-midi-key={key.note}
                        role="img"
                        aria-label={`${noteName(key.note)}${active ? ', playing' : ''}`}
                        title={`${noteName(key.note)} · MIDI ${key.note}`}>
                        {#if keyWidth >= 1 && (active || key.note % 12 === 0)}
                            <span
                                class="text-keyboard-black/65 absolute right-0 bottom-2 left-0 text-center font-mono text-xs font-semibold">
                                {noteName(key.note)}
                            </span>
                        {/if}
                    </div>
                {/each}
            </div>

            {#each blackKeys as key (key.note)}
                {@const active = notesByKey.has(key.note)}
                {@const keyNotes = notesByKey.get(key.note) ?? []}
                <div
                    class={`absolute top-0 z-10 h-24 rounded-b-md border border-t-0 shadow-md min-[56.25rem]:h-36 ${active ? 'border-keyboard-black/50 shadow-[0_0_0.75rem_var(--accent-glow-soft)]' : 'border-keyboard-black bg-keyboard-black'}`}
                    style:width={`${keyWidth * BLACK_KEY_WIDTH_RATIO}rem`}
                    style:background={active ? noteColor(keyNotes) : undefined}
                    style:left={`${(key.left / WHITE_KEY_WIDTH_REM) * keyWidth}rem`}
                    data-midi-key={key.note}
                    role="img"
                    aria-label={`${noteName(key.note)}${active ? ', playing' : ''}`}
                    title={`${noteName(key.note)} · MIDI ${key.note}`}>
                </div>
            {/each}
        </div>
    </div>
</section>

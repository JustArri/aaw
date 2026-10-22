<script lang="ts">
    import { notify } from '$lib/push-notifications.svelte';
    import {
        MAX_MIDI_EVENT_LOG_SIZE,
        midiEventLogSize,
        MIN_MIDI_EVENT_LOG_SIZE,
        setMidiEventLogSize
    } from '$lib/state/midi-events.svelte';
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

    const accentPresets = [
        { label: 'Blue', value: { r: 96, g: 199, b: 255 }, color: 'var(--color-synth-fluid)' },
        { label: 'Violet', value: { r: 166, g: 150, b: 255 }, color: 'var(--color-synth-sfizz)' },
        { label: 'Green', value: { r: 80, g: 212, b: 160 }, color: 'var(--color-synth-dummy)' },
        { label: 'Orange', value: { r: 255, g: 173, b: 114 }, color: 'var(--color-synth-warm)' },
        { label: 'Rose', value: { r: 255, g: 112, b: 136 }, color: 'var(--color-preset-rose)' }
    ];

    const themeModes: { value: ThemeMode; label: string }[] = [
        { value: 'light', label: 'Light' },
        { value: 'dark', label: 'Dark' },
        { value: 'auto', label: 'Auto' }
    ];

    function selectTheme(mode: ThemeMode) {
        applyTheme(mode);
        notify.success(`${mode === 'auto' ? 'Automatic' : mode} color mode selected.`, {
            title: 'Appearance'
        });
    }

    function selectAccent(value: Accent, label?: string) {
        setAccent(value);
        notify.success(label ? `${label} accent selected.` : 'Accent color updated.', {
            title: 'Appearance'
        });
    }

    function updateMidiEventLogSize(value: number) {
        setMidiEventLogSize(value);
        notify.success(`Keeping ${midiEventLogSize()} MIDI events.`, {
            title: 'MIDI event log'
        });
    }
</script>

<div class="grid w-full gap-3">
    <section class="border-border bg-surface-card overflow-hidden rounded-xl border">
        <header
            class="border-border [&_p]:text-foreground-muted flex min-h-16.75 items-center justify-between border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5 [&_h2]:m-0 [&_h2]:text-base min-[56.25rem]:[&_h2]:text-lg [&_p]:mt-1 [&_p]:mb-0 [&_p]:text-sm min-[56.25rem]:[&_p]:text-sm">
            <div>
                <h2>Appearance</h2>
                <p>Interface preferences are stored on this device.</p>
            </div>
            <span
                class="border-contrast/15 size-6.75 shrink-0 rounded-full border-2 shadow-[0_0_1rem_var(--accent-glow-soft)]"
                style:background={accentHex()}>
            </span>
        </header>
        <div class="border-border border-b p-3.75 min-[56.25rem]:p-5">
            <div>
                <span class="block text-sm font-semibold min-[56.25rem]:text-sm">Color mode</span>
                <small class="text-foreground-muted mt-1 block text-xs min-[56.25rem]:text-sm">
                    Use a light theme, a dark theme, or follow this device
                </small>
            </div>
            <div class="mt-3 grid max-w-95 grid-cols-3 gap-2" aria-label="Color mode">
                {#each themeModes as mode}
                    <button
                        class={`min-h-11 cursor-pointer rounded-lg border px-3 text-sm font-semibold transition-colors ${themeMode() === mode.value ? 'border-accent/50 bg-accent/10 text-accent' : 'border-border-strong bg-surface-inset text-foreground-strong hover:bg-surface-hover'}`}
                        type="button"
                        aria-pressed={themeMode() === mode.value}
                        onclick={() => selectTheme(mode.value)}>
                        {mode.label}
                    </button>
                {/each}
            </div>
        </div>
        <div class="p-3.75 min-[56.25rem]:p-5">
            <div>
                <span class="block text-sm font-semibold min-[56.25rem]:text-sm">Accent color</span>
                <small class="text-foreground-muted mt-1 block text-xs min-[56.25rem]:text-sm">
                    Choose any RGB color
                </small>
            </div>
            <label
                class="border-border-strong bg-surface-inset mt-3 flex w-full max-w-62.5 items-center gap-3 rounded-lg border px-2.5 py-2 font-mono text-sm">
                <input
                    class="h-6.75 w-9.5 border-0 bg-transparent p-0"
                    type="color"
                    value={accentHex()}
                    oninput={(event) => setAccentHex(event.currentTarget.value)}
                    onchange={() =>
                        notify.success('Accent color updated.', { title: 'Appearance' })} />
                <span>{accentHex().toUpperCase()}</span>
            </label>
            <div
                class="[&_span]:text-foreground-muted [&_input]:border-border-strong [&_input]:bg-surface-inset [&_input]:text-foreground mt-3 grid max-w-95 grid-cols-3 gap-2 [&_input]:min-h-9.5 [&_input]:w-full [&_input]:rounded-lg [&_input]:border [&_input]:p-2 [&_input]:text-sm [&_span]:mb-1 [&_span]:ml-0.5 [&_span]:block [&_span]:text-xs">
                {#each ['r', 'g', 'b'] as channel}<label>
                        <span>{channel.toUpperCase()}</span>
                        <input
                            type="number"
                            min="0"
                            max="255"
                            value={accent()[channel as keyof Accent]}
                            onchange={(event) =>
                                selectAccent({
                                    ...accent(),
                                    [channel]: Number(event.currentTarget.value)
                                })} />
                    </label>{/each}
            </div>
            <div class="mt-4 flex max-w-70 justify-between" aria-label="Accent color presets">
                {#each accentPresets as preset}<button
                        class="border-contrast/15 size-8 cursor-pointer rounded-full border-2"
                        type="button"
                        style:background={preset.color}
                        aria-label={`Use ${preset.label} accent`}
                        onclick={() => selectAccent(preset.value, preset.label)}>
                    </button>{/each}
            </div>
        </div>
    </section>
    <section class="border-border bg-surface-card overflow-hidden rounded-xl border">
        <header
            class="border-border [&_p]:text-foreground-muted flex min-h-16.75 items-center border-b px-3.75 py-3 min-[56.25rem]:min-h-19 min-[56.25rem]:px-5 [&_h2]:m-0 [&_h2]:text-base min-[56.25rem]:[&_h2]:text-lg [&_p]:mt-1 [&_p]:mb-0 [&_p]:text-sm min-[56.25rem]:[&_p]:text-sm">
            <div>
                <h2>MIDI</h2>
                <p>MIDI monitoring preferences are stored on this device.</p>
            </div>
        </header>
        <div class="flex items-center justify-between gap-5 p-3.75 min-[56.25rem]:p-5">
            <div>
                <label
                    class="block text-sm font-semibold min-[56.25rem]:text-sm"
                    for="midi-event-log-size">
                    Event log size
                </label>
                <small class="text-foreground-muted mt-1 block text-xs min-[56.25rem]:text-sm">
                    Maximum number of recent MIDI events retained
                </small>
            </div>
            <div class="flex shrink-0 items-center gap-2">
                <input
                    class="border-border-strong bg-surface-inset text-foreground h-9 w-20 rounded-lg border px-2.5 text-right font-mono text-sm tabular-nums"
                    id="midi-event-log-size"
                    type="number"
                    min={MIN_MIDI_EVENT_LOG_SIZE}
                    max={MAX_MIDI_EVENT_LOG_SIZE}
                    step="1"
                    value={midiEventLogSize()}
                    onchange={(event) =>
                        updateMidiEventLogSize(Number(event.currentTarget.value))} />
                <span class="text-foreground-muted text-sm">events</span>
            </div>
        </div>
    </section>
</div>

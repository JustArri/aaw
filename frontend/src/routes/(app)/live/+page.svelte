<script lang="ts">
    import Icon from '@iconify/svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import { icons } from '$lib/icons';

    type SynthKind = 'Fluid' | 'Sfizz' | 'Dummy';
    type SynthId = number;

    type Instrument = {
        id: number;
        name: string;
        enabled: boolean;
        synthId: SynthId | null;
        channelFilter: string;
        ccFilter: string;
        keySelection: string;
        keyMapping: string;
        transposition: number;
        useGlobalTransposition: boolean;
        color: string;
    };

    type Synth = {
        id: number;
        kind: SynthKind;
        name: string;
        path?: string;
        program?: string;
        metadata: string;
        color: string;
    };

    let synths = $state<Synth[]>([
        {
            id: 2,
            kind: 'Fluid',
            name: 'Vintage Dreams',
            path: 'soundfonts/VintageDreamsWaves-v2.sf2',
            program: 'Warm Pad · Bank 0 / 89',
            metadata: '1 bank · 128 presets',
            color: 'var(--color-synth-fluid)'
        },
        {
            id: 1,
            kind: 'Fluid',
            name: 'GeneralUser GS',
            path: 'soundfonts/GeneralUser-GS.sf2',
            program: 'Synth Bass 2 · Bank 0 / 39',
            metadata: '2 banks · 235 presets',
            color: 'var(--color-synth-warm)'
        },
        {
            id: 3,
            kind: 'Sfizz',
            name: 'Stage Piano',
            path: 'instruments/stage-piano/stage-piano.sfz',
            metadata: '52 regions · A0–C8',
            color: 'var(--color-synth-sfizz)'
        },
        {
            id: 0,
            kind: 'Dummy',
            name: 'Test oscillator',
            metadata: 'Diagnostic synth',
            color: 'var(--color-synth-dummy)'
        }
    ]);

    let instruments = $state<Instrument[]>([
        {
            id: 0,
            name: 'Ambient Pad',
            enabled: true,
            synthId: 0,
            channelFilter: 'Channel 1',
            ccFilter: 'All controllers',
            keySelection: 'All keys',
            keyMapping: 'Identity',
            transposition: 0,
            useGlobalTransposition: true,
            color: 'var(--color-synth-fluid)'
        },
        {
            id: 1,
            name: 'Main Piano',
            enabled: true,
            synthId: 2,
            channelFilter: 'Channel 2',
            ccFilter: 'Deny 64',
            keySelection: 'A0 – C8',
            keyMapping: 'Identity',
            transposition: 0,
            useGlobalTransposition: true,
            color: 'var(--color-synth-sfizz)'
        },
        {
            id: 2,
            name: 'Bass Split',
            enabled: true,
            synthId: 1,
            channelFilter: 'Channel 1',
            ccFilter: 'All controllers',
            keySelection: 'C0 – B2',
            keyMapping: 'Identity',
            transposition: -12,
            useGlobalTransposition: false,
            color: 'var(--color-synth-warm)'
        },
        {
            id: 3,
            name: 'Unassigned instrument',
            enabled: false,
            synthId: null,
            channelFilter: 'All channels',
            ccFilter: 'All controllers',
            keySelection: 'All keys',
            keyMapping: 'Identity',
            transposition: 0,
            useGlobalTransposition: true,
            color: 'var(--color-synth-unassigned)'
        }
    ]);

    let globalTransposition = $state(0);
    let globalGain = $state(-3);
    let activePreset = $state('Full setup');

    function synthById(id: SynthId | null) {
        if (id === null) {
            return undefined;
        }
        return synths.find((synth) => synth.id === id);
    }

    function toggleInstrument(id: number) {
        activePreset = '';
        instruments = instruments.map((instrument) =>
            instrument.id === id ? { ...instrument, enabled: !instrument.enabled } : instrument
        );
    }

    const livePresets = [
        { name: 'Full setup', description: 'All layers', enabled: [0, 1, 2] },
        { name: 'Keys + pad', description: 'Piano and ambience', enabled: [0, 1] },
        { name: 'Piano only', description: 'Main piano', enabled: [1] },
        { name: 'Silence', description: 'Disable all', enabled: [] }
    ];

    async function applyPreset(name: string, enabled: number[]) {
        activePreset = name;
        instruments = instruments.map((instrument) => ({
            ...instrument,
            enabled: enabled.includes(instrument.id)
        }));
    }

    async function setGlobalTransposition(value: number) {
        globalTransposition = value;
    }

    async function setGlobalGain(value: number) {
        globalGain = value;
    }
</script>

<div class="grid gap-3 min-[37.5rem]:grid-cols-2 min-[56.25rem]:grid-cols-1 min-[56.25rem]:gap-3.5">
    <div
        class="contents min-[56.25rem]:row-start-2 min-[56.25rem]:grid min-[56.25rem]:grid-cols-2 min-[56.25rem]:gap-3.5">
        <section class="order-1 min-w-0 min-[56.25rem]:order-0" aria-label="Global transposition">
            <NumericControl
                label="Global transposition"
                value={globalTransposition}
                unit="semitones"
                smallStep={1}
                largeStep={12}
                resetValue={0}
                min={-48}
                max={48}
                wideValue
                onchange={setGlobalTransposition} />
        </section>
        <section class="order-4 min-w-0 min-[56.25rem]:order-0" aria-label="Global gain">
            <NumericControl
                label="Global gain"
                value={globalGain}
                unit="dB"
                smallStep={0.5}
                largeStep={5}
                resetValue={0}
                min={-60}
                max={12}
                precision={1}
                onchange={setGlobalGain} />
        </section>
    </div>

    <div class="contents min-[56.25rem]:row-start-1 min-[56.25rem]:grid min-[56.25rem]:gap-3.5">
        <section
            class="border-border bg-surface-card order-2 overflow-hidden rounded-xl border min-[56.25rem]:order-0">
            <header
                class="border-border [&_small]:text-foreground-muted [&_b]:bg-surface-badge [&_b]:text-foreground-strong flex min-h-13.5 items-center justify-between border-b px-3.25 min-[56.25rem]:min-h-14.75 min-[56.25rem]:px-3.75 [&_b]:grid [&_b]:h-5.75 [&_b]:min-w-6.75 [&_b]:place-items-center [&_b]:rounded-md [&_b]:font-mono [&_b]:text-xs [&_b]:font-semibold min-[56.25rem]:[&_b]:text-sm [&_small]:mt-1 [&_small]:block [&_small]:text-xs min-[56.25rem]:[&_small]:text-sm [&_span]:block [&_span]:text-sm [&_span]:font-bold min-[56.25rem]:[&_span]:text-sm">
                <div>
                    <span>Scene presets</span>
                    <small>Instrument states saved with this project</small>
                </div>
                <b class="min-w-0! px-1.5 text-center font-sans! leading-none tabular-nums">
                    <span>{livePresets.length}</span>
                </b>
            </header>
            <div class="grid grid-cols-2 gap-1.75 p-2.25 min-[56.25rem]:grid-cols-4">
                {#each livePresets as preset, index}
                    <button
                        class={`bg-surface-item relative grid min-h-18.75 cursor-pointer grid-cols-[1.375rem_minmax(0,1fr)] content-center items-start gap-1.75 overflow-hidden rounded-lg border px-2.25 py-2.75 text-left ${activePreset === preset.name ? 'border-accent/50 bg-accent/9 shadow-[inset_0.1875rem_0_0_var(--accent)]' : 'border-border-strong'}`}
                        type="button"
                        onclick={() => applyPreset(preset.name, preset.enabled)}>
                        <small
                            class={`font-mono text-xs min-[56.25rem]:text-sm ${activePreset === preset.name ? 'text-accent' : 'text-foreground-index'}`}>
                            {String(index + 1).padStart(2, '0')}
                        </small>
                        <span class="min-w-0">
                            <strong
                                class={`block overflow-hidden text-sm text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm ${activePreset === preset.name ? 'text-accent' : ''}`}>
                                {preset.name}
                            </strong>
                            <em
                                class="text-foreground-muted mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap not-italic min-[56.25rem]:text-sm">
                                {preset.description}
                            </em>
                        </span>
                        <i
                            class={`absolute right-2 bottom-2 size-1.25 rounded-full ${activePreset === preset.name ? 'bg-accent shadow-[0_0_0.4375rem_var(--accent-glow)]' : 'bg-indicator-off'}`}>
                        </i>
                    </button>
                {/each}
            </div>
        </section>

        <section
            class="border-border bg-surface-card order-3 overflow-hidden rounded-xl border min-[56.25rem]:order-0">
            <header
                class="border-border [&_small]:text-foreground-muted [&_b]:bg-surface-badge [&_b]:text-foreground-strong flex min-h-13.5 items-center justify-between border-b px-3.25 min-[56.25rem]:min-h-14.75 min-[56.25rem]:px-3.75 [&_b]:grid [&_b]:h-5.75 [&_b]:min-w-6.75 [&_b]:place-items-center [&_b]:rounded-md [&_b]:font-mono [&_b]:text-xs [&_b]:font-semibold min-[56.25rem]:[&_b]:text-sm [&_small]:mt-1 [&_small]:block [&_small]:text-xs min-[56.25rem]:[&_small]:text-sm [&_span]:block [&_span]:text-sm [&_span]:font-bold min-[56.25rem]:[&_span]:text-sm">
                <div>
                    <span>Instruments</span>
                    <small>Tap an instrument to toggle it</small>
                </div>
                <b class="min-w-0! px-1.5 text-center font-sans! leading-none tabular-nums">
                    <span>
                        {instruments.filter((item) => item.enabled).length} / {instruments.length}
                    </span>
                </b>
            </header>
            <div class="bg-border grid gap-px">
                {#each instruments as instrument}
                    {@const assignedSynth = synthById(instrument.synthId)}
                    <button
                        class={`grid min-h-14.5 cursor-pointer grid-cols-[0.1875rem_minmax(0,1fr)_2.125rem] items-center gap-2.5 border-0 px-2.5 py-1.75 text-left min-[56.25rem]:min-h-15.5 min-[56.25rem]:px-3.25 ${instrument.enabled ? 'bg-surface-item-active' : 'bg-surface-disabled'}`}
                        type="button"
                        aria-pressed={instrument.enabled}
                        onclick={() => toggleInstrument(instrument.id)}>
                        <i
                            class={`h-7.25 rounded ${instrument.enabled ? '' : 'opacity-35 grayscale'}`}
                            style:background={instrument.color}>
                        </i>
                        <span class="min-w-0">
                            <strong
                                class={`block overflow-hidden text-sm font-semibold text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm ${instrument.enabled ? 'text-foreground' : 'text-foreground-disabled'}`}>
                                {instrument.name}
                            </strong>
                            <small
                                class="text-foreground-caption mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm">
                                {assignedSynth?.name ?? 'No synth assigned'}
                            </small>
                        </span>
                        <b
                            class={`grid h-8 place-items-center rounded-lg border ${instrument.enabled ? 'border-accent/30 bg-accent/10 text-accent' : 'border-border-strong text-foreground-off'}`}>
                            <Icon icon={icons.power} width="0.9375rem" />
                        </b>
                    </button>
                {/each}
            </div>
        </section>
    </div>
</div>

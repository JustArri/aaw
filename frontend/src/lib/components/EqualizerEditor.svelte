<script lang="ts">
    import Icon from '@iconify/svelte';
    import Button from '$lib/components/Button.svelte';
    import SelectDropdown from '$lib/components/SelectDropdown.svelte';
    import { equalizerPresets } from '$lib/content/equalizer-presets';
    import type { EqualizerBand } from '$lib/generated/models';

    type BandKind = 'LowShelf' | 'Peaking' | 'HighShelf';
    type Biquad = { b0: number; b1: number; b2: number; a1: number; a2: number };

    let {
        bands = $bindable(),
        maxBands = 16,
        sampleRate = 48_000
    }: {
        bands: EqualizerBand[];
        maxBands?: number;
        sampleRate?: number;
    } = $props();

    let selectedPresetIndex = $derived.by(() => {
        const serialized = JSON.stringify(bands);
        return equalizerPresets.findIndex(
            (preset) => JSON.stringify(preset.definition.bands) === serialized
        );
    });

    function clone<T>(value: T): T {
        return JSON.parse(JSON.stringify(value)) as T;
    }

    function parameters(band: EqualizerBand) {
        return band.data;
    }

    function replaceBand(index: number, kind: BandKind) {
        bands[index] = {
            type: kind,
            data: clone(parameters(bands[index]))
        } as EqualizerBand;
    }

    function applyPreset(index: number) {
        const preset = equalizerPresets[index];
        if (preset) {
            bands = clone(preset.definition.bands.slice(0, maxBands));
        }
    }

    function addBand() {
        if (bands.length < maxBands) {
            bands.push({
                type: 'Peaking',
                data: { frequency: 1000, gain_db: 0, resonance: 0.707 }
            });
        }
    }

    function coefficients(band: EqualizerBand): Biquad | null {
        const data = parameters(band);
        if (data.frequency <= 0 || data.resonance <= 0 || !Number.isFinite(data.gain_db)) {
            return null;
        }
        const frequency = Math.min(data.frequency, sampleRate * 0.499);
        const omega = (Math.PI * 2 * frequency) / sampleRate;
        const sin = Math.sin(omega);
        const cos = Math.cos(omega);
        const alpha = sin / (2 * data.resonance);
        const amplitude = 10 ** (data.gain_db / 40);
        let values: [number, number, number, number, number, number];
        switch (band.type) {
            case 'Peaking':
                values = [
                    1 + alpha * amplitude,
                    -2 * cos,
                    1 - alpha * amplitude,
                    1 + alpha / amplitude,
                    -2 * cos,
                    1 - alpha / amplitude
                ];
                break;
            case 'LowShelf': {
                const term = 2 * Math.sqrt(amplitude) * alpha;
                values = [
                    amplitude * (amplitude + 1 - (amplitude - 1) * cos + term),
                    2 * amplitude * (amplitude - 1 - (amplitude + 1) * cos),
                    amplitude * (amplitude + 1 - (amplitude - 1) * cos - term),
                    amplitude + 1 + (amplitude - 1) * cos + term,
                    -2 * (amplitude - 1 + (amplitude + 1) * cos),
                    amplitude + 1 + (amplitude - 1) * cos - term
                ];
                break;
            }
            case 'HighShelf': {
                const term = 2 * Math.sqrt(amplitude) * alpha;
                values = [
                    amplitude * (amplitude + 1 + (amplitude - 1) * cos + term),
                    -2 * amplitude * (amplitude - 1 + (amplitude + 1) * cos),
                    amplitude * (amplitude + 1 + (amplitude - 1) * cos - term),
                    amplitude + 1 - (amplitude - 1) * cos + term,
                    2 * (amplitude - 1 - (amplitude + 1) * cos),
                    amplitude + 1 - (amplitude - 1) * cos - term
                ];
                break;
            }
        }
        const [b0, b1, b2, a0, a1, a2] = values;
        return { b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: a1 / a0, a2: a2 / a0 };
    }

    function curve() {
        const width = 300;
        const height = 96;
        const filters = bands.map(coefficients).filter((item): item is Biquad => item !== null);
        return Array.from({ length: 121 }, (_, index) => {
            const x = (index / 120) * width;
            const frequency = 20 * 1000 ** (index / 120);
            const omega = (Math.PI * 2 * Math.min(frequency, sampleRate * 0.499)) / sampleRate;
            let gain = 0;
            for (const filter of filters) {
                const numeratorReal =
                    filter.b0 + filter.b1 * Math.cos(omega) + filter.b2 * Math.cos(2 * omega);
                const numeratorImaginary =
                    -filter.b1 * Math.sin(omega) - filter.b2 * Math.sin(2 * omega);
                const denominatorReal =
                    1 + filter.a1 * Math.cos(omega) + filter.a2 * Math.cos(2 * omega);
                const denominatorImaginary =
                    -filter.a1 * Math.sin(omega) - filter.a2 * Math.sin(2 * omega);
                gain +=
                    10 *
                    Math.log10(
                        Math.max(
                            Number.EPSILON,
                            (numeratorReal ** 2 + numeratorImaginary ** 2) /
                                (denominatorReal ** 2 + denominatorImaginary ** 2)
                        )
                    );
            }
            const y = height / 2 - (Math.max(-24, Math.min(24, gain)) / 48) * height;
            return `${index === 0 ? 'M' : 'L'}${x.toFixed(2)},${y.toFixed(2)}`;
        }).join(' ');
    }
</script>

<div class="border-border grid gap-4 border-t p-3 min-[37.5rem]:p-4">
    <div class="grid gap-2">
        <span class="text-foreground-muted text-xs font-semibold uppercase">Preset</span>
        <SelectDropdown
            class="min-[37.5rem]:max-w-96"
            value={String(selectedPresetIndex)}
            options={[
                { value: '-1', label: 'Custom settings', disabled: true },
                ...equalizerPresets.map((preset, index) => ({
                    value: String(index),
                    label: preset.name
                }))
            ]}
            ariaLabel="Apply equalizer preset"
            onchange={(value) => applyPreset(Number(value))} />
        <small class="text-foreground-muted">
            {selectedPresetIndex >= 0
                ? `Selected preset: ${equalizerPresets[selectedPresetIndex].name}`
                : 'Custom settings'}
            · Applying a preset replaces the current bands.
        </small>
    </div>
    <svg
        class="border-border bg-surface-inset h-44 w-full rounded-xl border min-[56.25rem]:h-56"
        viewBox="0 0 300 96"
        preserveAspectRatio="none"
        role="img"
        aria-label="Equalizer frequency response">
        <path d="M0 24H300M0 48H300M0 72H300" stroke="var(--border)" />
        <path
            d={curve()}
            fill="none"
            stroke="var(--accent)"
            stroke-width="2"
            vector-effect="non-scaling-stroke" />
    </svg>
    <div class="grid gap-2">
        {#each bands as band, index (band)}
            {@const data = parameters(band)}
            <div
                class="border-border bg-surface-inset grid grid-cols-[minmax(0,1fr)_2.5rem] gap-2 rounded-lg border p-2">
                <div
                    class="grid min-w-0 grid-cols-3 gap-2 min-[56.25rem]:grid-cols-[minmax(8rem,1fr)_repeat(3,minmax(7rem,1fr))]">
                    <SelectDropdown
                        class="col-span-3 min-[56.25rem]:col-span-1"
                        compact
                        value={band.type}
                        options={[
                            { value: 'LowShelf', label: 'Low shelf' },
                            { value: 'Peaking', label: 'Peaking' },
                            { value: 'HighShelf', label: 'High shelf' }
                        ]}
                        onchange={(value) => replaceBand(index, value as BandKind)}
                        ariaLabel="Equalizer band type" />
                    {#each [['Hz', 'frequency', 1], ['dB', 'gain_db', 0.1], ['Q', 'resonance', 0.01]] as field}
                        <label class="grid min-w-0 gap-1 text-[0.6875rem]">
                            <span class="text-foreground-muted">{field[0]}</span>
                            <input
                                class="border-border-strong bg-surface-card h-9 min-w-0 rounded-md border px-2 font-mono text-xs"
                                type="number"
                                min={field[1] === 'gain_db' ? undefined : 0.001}
                                step={field[2]}
                                bind:value={data[field[1] as keyof typeof data]} />
                        </label>
                    {/each}
                </div>
                <Button
                    variant="danger-ghost"
                    size="md"
                    square
                    class="self-end"
                    type="button"
                    aria-label="Remove equalizer band"
                    onclick={() => bands.splice(index, 1)}>
                    <Icon icon="lucide:x" width="0.875rem" />
                </Button>
            </div>
        {:else}
            <p class="text-foreground-muted m-0 px-1 py-2 text-center text-xs">
                No equalizer bands
            </p>
        {/each}
    </div>
    <Button
        variant="secondary"
        size="md"
        class="text-foreground-muted"
        type="button"
        disabled={bands.length >= maxBands}
        onclick={addBand}>
        Add band
    </Button>
</div>

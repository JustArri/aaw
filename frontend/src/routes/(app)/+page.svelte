<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import { decibelsToLinearGain, linearGainToDecibels } from '$lib/gain';
    import type { SynthId } from '$lib/generated/models';
    import { icons } from '$lib/icons';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject } from '$lib/state/active-project.svelte';
    import {
        fetchInstruments,
        instrumentGlobalTransposition,
        instrumentSlots,
        setInstrumentEnabled,
        setInstrumentGlobalTransposition
    } from '$lib/state/instruments.svelte';
    import {
        fetchSynths,
        setGlobalGain,
        synths,
        SynthKind,
        type SynthEntry
    } from '$lib/state/synths.svelte';
    import {
        activateUserPreset,
        cloneUserPresets,
        fetchUserPresets,
        saveUserPresets,
        userPresets
    } from '$lib/state/user-presets.svelte';

    let loading = $state(true);
    let togglingInstrument = $state<number | null>(null);
    let activatingPreset = $state<number | null>(null);
    let creatingPreset = $state(false);

    let instruments = $derived(
        Object.entries(instrumentSlots())
            .map(([id, slot]) => ({ id: Number(id), slot }))
            .sort((left, right) => left.id - right.id)
    );
    let project = $derived(activeProject());
    let globalGainDb = $derived(linearGainToDecibels(project?.synth_graph.global_gain ?? 1));
    let availableSynths = $derived(synths());
    let enabledInstrumentIds = $derived(
        instruments.filter(({ slot }) => slot.enabled).map(({ id }) => id)
    );
    let activePresetIndex = $derived(
        userPresets().entries.findIndex((preset) => {
            const presetIds = [...preset.instruments].sort((left, right) => left - right);
            return (
                presetIds.length === enabledInstrumentIds.length &&
                presetIds.every((id, index) => id === enabledInstrumentIds[index])
            );
        })
    );

    onMount(async () => {
        try {
            await Promise.all([fetchInstruments(), fetchSynths(), fetchUserPresets()]);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: 'Load live controls'
            });
        } finally {
            loading = false;
        }
    });

    function synthEntry(id: SynthId | null) {
        if (id === null) {
            return undefined;
        }
        return availableSynths.find((entry) => entry.id === id);
    }

    function synthName(entry: SynthEntry | undefined) {
        return (
            entry?.synth.name.trim() ||
            (entry ? `Unnamed ${entry.kind.toLowerCase()} synth #${entry.id}` : 'No synth assigned')
        );
    }

    function presetName(index: number, name: string) {
        return name.trim() || `Unnamed preset #${index + 1}`;
    }

    function synthColor(id: SynthId | null) {
        switch (synthEntry(id)?.kind) {
            case SynthKind.Fluid:
                return 'var(--color-synth-fluid)';
            case SynthKind.Sfizz:
                return 'var(--color-synth-sfizz)';
            case SynthKind.Dummy:
                return 'var(--color-synth-dummy)';
            default:
                return 'var(--color-synth-unassigned)';
        }
    }

    async function toggleInstrument(id: number, enabled: boolean) {
        if (togglingInstrument !== null || activatingPreset !== null || creatingPreset) {
            return;
        }
        togglingInstrument = id;
        try {
            await setInstrumentEnabled(id, enabled);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: enabled ? 'Enable instrument' : 'Disable instrument'
            });
        } finally {
            togglingInstrument = null;
        }
    }

    async function applyPreset(index: number) {
        if (activatingPreset !== null || togglingInstrument !== null || creatingPreset) {
            return;
        }
        activatingPreset = index;
        try {
            await activateUserPreset(index);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: `Activate ${userPresets().entries[index]?.name ?? 'preset'}`
            });
        } finally {
            activatingPreset = null;
        }
    }

    async function createPreset() {
        if (
            creatingPreset ||
            activatingPreset !== null ||
            togglingInstrument !== null ||
            activePresetIndex !== -1
        ) {
            return;
        }
        creatingPreset = true;
        const nextPresetNumber = userPresets().entries.length + 1;
        try {
            const updated = cloneUserPresets(userPresets());
            updated.entries.push({
                name: `Preset ${nextPresetNumber}`,
                instruments: [...enabledInstrumentIds]
            });
            await saveUserPresets(updated);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: 'Create user preset'
            });
        } finally {
            creatingPreset = false;
        }
    }

    async function setGlobalGainDb(value: number) {
        await setGlobalGain(decibelsToLinearGain(value));
        return value;
    }
</script>

<div class="grid gap-3 min-[37.5rem]:grid-cols-2 min-[37.5rem]:items-start min-[56.25rem]:gap-3.5">
    <div
        class="contents min-[37.5rem]:order-0 min-[37.5rem]:col-start-2 min-[37.5rem]:row-start-2 min-[37.5rem]:grid min-[37.5rem]:content-start min-[37.5rem]:gap-3 min-[56.25rem]:gap-3.5">
        <section class="order-1 min-w-0 min-[56.25rem]:order-0" aria-label="Global transposition">
            <NumericControl
                label="Global transposition"
                value={instrumentGlobalTransposition()}
                unit="semitones"
                smallStep={1}
                largeStep={12}
                resetValue={0}
                min={-128}
                max={127}
                wideValue
                disabled={loading}
                onchange={setInstrumentGlobalTransposition} />
        </section>
        <section class="order-4 min-w-0 min-[56.25rem]:order-0" aria-label="Global gain">
            <NumericControl
                label="Global gain"
                value={globalGainDb}
                unit="dB"
                smallStep={0.5}
                largeStep={5}
                resetValue={0}
                min={-100}
                max={100}
                precision={1}
                disabled={loading || !project}
                onchange={setGlobalGainDb} />
        </section>
    </div>

    <div class="contents">
        <section
            class="border-border bg-surface-card order-2 overflow-hidden rounded-xl border min-[37.5rem]:order-0 min-[37.5rem]:col-span-2 min-[37.5rem]:row-start-1">
            <header
                class="border-border flex min-h-13.5 items-center justify-between border-b px-3.25 min-[56.25rem]:min-h-14.75 min-[56.25rem]:px-3.75">
                <div>
                    <span class="block text-sm font-bold">User presets</span>
                </div>
                <div class="flex items-center gap-2">
                    <b
                        class="bg-surface-badge text-foreground-strong grid h-5.75 min-w-0! place-items-center rounded-md px-1.5 text-center font-sans! text-xs leading-none font-semibold tabular-nums min-[56.25rem]:text-sm">
                        <span>{userPresets().entries.length}</span>
                    </b>
                    <button
                        class="border-border-strong bg-surface-button text-foreground hover:bg-surface-hover grid size-8.5 cursor-pointer place-items-center rounded-lg border transition-colors disabled:cursor-not-allowed disabled:opacity-50"
                        type="button"
                        aria-label={activePresetIndex !== -1
                            ? 'Current selection already matches a preset'
                            : 'Create preset from enabled instruments'}
                        title={activePresetIndex !== -1
                            ? 'Current selection already matches a preset'
                            : 'Create preset from enabled instruments'}
                        disabled={loading ||
                            creatingPreset ||
                            activatingPreset !== null ||
                            togglingInstrument !== null ||
                            activePresetIndex !== -1}
                        onclick={() => void createPreset()}>
                        <Icon
                            icon={creatingPreset ? 'lucide:loader-circle' : 'lucide:plus'}
                            width="1rem"
                            class={creatingPreset ? 'animate-spin' : ''} />
                    </button>
                </div>
            </header>
            {#if loading}
                <div class="text-foreground-muted grid min-h-24 place-items-center text-sm">
                    Loading presets…
                </div>
            {:else if userPresets().entries.length === 0}
                <div class="text-foreground-muted grid min-h-24 place-items-center p-4 text-sm">
                    No user presets configured.
                </div>
            {:else}
                <div class="grid grid-cols-2 gap-1.75 p-2.25 min-[37.5rem]:grid-cols-4">
                    {#each userPresets().entries as preset, index}
                        <button
                            class={`bg-surface-item relative grid min-h-18.75 cursor-pointer grid-cols-[1.375rem_minmax(0,1fr)] content-center items-start gap-1.75 overflow-hidden rounded-lg border px-2.25 py-2.75 text-left transition-colors disabled:cursor-wait ${activePresetIndex === index ? 'border-accent/50 bg-accent/9 shadow-[inset_0.1875rem_0_0_var(--accent)]' : 'border-border-strong hover:border-accent/30'}`}
                            type="button"
                            disabled={activatingPreset !== null ||
                                togglingInstrument !== null ||
                                creatingPreset}
                            onclick={() => void applyPreset(index)}>
                            <small
                                class={`font-mono text-xs min-[56.25rem]:text-sm ${activePresetIndex === index ? 'text-accent' : 'text-foreground-index'}`}>
                                {String(index + 1).padStart(2, '0')}
                            </small>
                            <span class="min-w-0">
                                <strong
                                    class={`block overflow-hidden text-sm text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm ${preset.name.trim() ? (activePresetIndex === index ? 'text-accent' : 'text-foreground') : 'text-foreground-muted italic'}`}>
                                    {presetName(index, preset.name)}
                                </strong>
                                <em
                                    class="text-foreground-muted mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap not-italic min-[56.25rem]:text-sm">
                                    {preset.instruments.length}
                                    {preset.instruments.length === 1 ? 'instrument' : 'instruments'}
                                </em>
                            </span>
                            {#if activatingPreset === index}
                                <Icon
                                    icon="lucide:loader-circle"
                                    width="0.875rem"
                                    class="text-accent absolute right-2 bottom-2 animate-spin" />
                            {:else}<i
                                    class={`absolute right-2 bottom-2 size-1.25 rounded-full ${activePresetIndex === index ? 'bg-accent shadow-[0_0_0.4375rem_var(--accent-glow)]' : 'bg-indicator-off'}`}>
                                </i>{/if}
                        </button>
                    {/each}
                </div>
            {/if}
        </section>

        <section
            class="border-border bg-surface-card order-3 overflow-hidden rounded-xl border min-[37.5rem]:order-0 min-[37.5rem]:col-start-1 min-[37.5rem]:row-start-2">
            <header
                class="border-border flex min-h-13.5 items-center justify-between border-b px-3.25 min-[56.25rem]:min-h-14.75 min-[56.25rem]:px-3.75">
                <div>
                    <span class="block text-sm font-bold">Instruments</span>
                </div>
                <b
                    class="bg-surface-badge text-foreground-strong grid h-5.75 min-w-0! place-items-center rounded-md px-1.5 text-center font-sans! text-xs leading-none font-semibold tabular-nums min-[56.25rem]:text-sm">
                    <span>{enabledInstrumentIds.length} / {instruments.length}</span>
                </b>
            </header>
            {#if loading}
                <div class="text-foreground-muted grid min-h-24 place-items-center text-sm">
                    Loading instruments…
                </div>
            {:else if instruments.length === 0}
                <div class="text-foreground-muted grid min-h-24 place-items-center p-4 text-sm">
                    No instruments configured.
                </div>
            {:else}
                <div class="bg-border grid gap-px">
                    {#each instruments as instrument (instrument.id)}
                        {@const assignedSynth = synthEntry(instrument.slot.synth_id)}
                        <button
                            class={`grid min-h-14.5 cursor-pointer grid-cols-[0.1875rem_minmax(0,1fr)_2.125rem] items-center gap-2.5 border-0 px-2.5 py-1.75 text-left disabled:cursor-wait min-[56.25rem]:min-h-15.5 min-[56.25rem]:px-3.25 ${instrument.slot.enabled ? 'bg-surface-item-active' : 'bg-surface-disabled'}`}
                            type="button"
                            aria-pressed={instrument.slot.enabled}
                            disabled={togglingInstrument === instrument.id}
                            onclick={() =>
                                void toggleInstrument(instrument.id, !instrument.slot.enabled)}>
                            <i
                                class={`h-7.25 rounded ${instrument.slot.enabled ? '' : 'opacity-35 grayscale'}`}
                                style:background={synthColor(instrument.slot.synth_id)}>
                            </i>
                            <span class="min-w-0">
                                <strong
                                    class={`block overflow-hidden text-sm font-semibold text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm ${instrument.slot.name.trim() ? (instrument.slot.enabled ? 'text-foreground' : 'text-foreground-disabled') : 'text-foreground-muted italic'}`}>
                                    {instrument.slot.name.trim() ||
                                        `Unnamed instrument #${instrument.id + 1}`}
                                </strong>
                                <small
                                    class="text-foreground-caption mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm">
                                    {synthName(assignedSynth)}
                                </small>
                            </span>
                            <b
                                class={`grid h-8 place-items-center rounded-lg border ${instrument.slot.enabled ? 'border-accent/30 bg-accent/10 text-accent' : 'border-border-strong text-foreground-off'}`}>
                                <Icon
                                    icon={togglingInstrument === instrument.id
                                        ? 'lucide:loader-circle'
                                        : icons.power}
                                    width="0.9375rem"
                                    class={togglingInstrument === instrument.id
                                        ? 'animate-spin'
                                        : ''} />
                            </b>
                        </button>
                    {/each}
                </div>
            {/if}
        </section>
    </div>
</div>

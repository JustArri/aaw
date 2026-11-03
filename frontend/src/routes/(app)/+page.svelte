<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onDestroy, onMount } from 'svelte';
    import NumericControl from '$lib/components/NumericControl.svelte';
    import ProjectTransport from '$lib/components/ProjectTransport.svelte';
    import { decibelsToLinearGain, linearGainToDecibels } from '$lib/gain';
    import type { UserPresetAction } from '$lib/generated/models';
    import { icons } from '$lib/icons';
    import { notify } from '$lib/push-notifications.svelte';
    import { activeProject } from '$lib/state/active-project.svelte';
    import {
        fetchInstruments,
        instrumentGlobalTransposition,
        setInstrumentGlobalTransposition
    } from '$lib/state/instruments.svelte';
    import {
        controllerEntries,
        fetchSynths,
        setControllerEnabled,
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
    let activationAnimationPreset = $state<number | null>(null);
    let creatingPreset = $state(false);
    let activationAnimationTimer: ReturnType<typeof setTimeout> | undefined;

    const ACTIVATION_GLOW_DURATION = 320;

    let instruments = $derived(controllerEntries().sort((left, right) => left.id - right.id));
    let project = $derived(activeProject());
    let globalGainDb = $derived(linearGainToDecibels(project?.instrument_graph.global_gain ?? 1));
    let availableSynths = $derived(synths());
    let enabledControllerCount = $derived(
        instruments.filter(({ controller }) => controller.data.enabled).length
    );

    onDestroy(() => clearTimeout(activationAnimationTimer));

    function activationGlow(node: HTMLSpanElement) {
        const animation = node.animate(
            [
                {
                    opacity: 1,
                    boxShadow: 'inset 0 0 0.75rem var(--accent-glow), 0 0 1rem var(--accent-glow)'
                },
                {
                    opacity: 0,
                    boxShadow:
                        'inset 0 0 0.125rem var(--accent-glow-soft), 0 0 0.25rem var(--accent-glow-soft)'
                }
            ],
            {
                duration: ACTIVATION_GLOW_DURATION,
                easing: 'cubic-bezier(0.16, 1, 0.3, 1)',
                fill: 'forwards'
            }
        );
        return {
            destroy() {
                animation.cancel();
            }
        };
    }

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

    function synthEntry(id: number | null) {
        if (id === null) {
            return undefined;
        }
        return availableSynths.find((entry) => entry.id === id);
    }

    function synthName(entry: SynthEntry | undefined) {
        return (
            entry?.synth.name.trim() ||
            (entry ? `${entry.kind} instrument #${entry.id + 1}` : 'Instrument')
        );
    }

    function soundFontProgramName(entry: SynthEntry | undefined) {
        if (entry?.kind !== SynthKind.Fluid) {
            return undefined;
        }
        const program = entry.synth.program;
        const name = entry.synth.info.banks
            .find((bank) => bank.number === program.bank)
            ?.presets.find((preset) => preset.number === program.preset_id)
            ?.name.trim();
        return name || `Bank ${program.bank} / Program ${program.preset_id}`;
    }

    function controllerName(entry: (typeof instruments)[number]) {
        const type = entry.controller.type === 'MidiInput' ? 'MIDI input' : 'Drum machine';
        const name = entry.controller.data.name.trim();
        return name ? `${type}: ${name}` : type;
    }

    function presetName(index: number, name: string) {
        return name.trim() || `Unnamed preset #${index + 1}`;
    }

    function currentStateActions(): UserPresetAction[] {
        return [
            'DisableAllInstrumentControllers',
            ...instruments
                .filter(({ controller }) => controller.data.enabled)
                .map((entry) => ({
                    SetInstrumentControllerEnabled: {
                        controller: {
                            instrument_id: entry.instrument_id,
                            controller_id: entry.controller_id
                        },
                        flag: true
                    }
                })),
            ...availableSynths.flatMap((entry) =>
                entry.kind === SynthKind.Fluid
                    ? [
                          {
                              SetSoundFontProgram: {
                                  instrument: entry.id,
                                  program: { ...entry.synth.program }
                              }
                          }
                      ]
                    : []
            )
        ];
    }

    function synthColor(id: number | null) {
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
            await setControllerEnabled(id, enabled);
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
        clearTimeout(activationAnimationTimer);
        activationAnimationPreset = null;
        activatingPreset = index;
        try {
            await activateUserPreset(index);
            activationAnimationPreset = index;
            activationAnimationTimer = setTimeout(() => {
                activationAnimationPreset = null;
            }, ACTIVATION_GLOW_DURATION + 30);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: `Activate ${userPresets().entries[index]?.name ?? 'preset'}`
            });
        } finally {
            activatingPreset = null;
        }
    }

    async function createPreset() {
        if (creatingPreset || activatingPreset !== null || togglingInstrument !== null) {
            return;
        }
        creatingPreset = true;
        const nextPresetNumber = userPresets().entries.length + 1;
        try {
            const updated = cloneUserPresets(userPresets());
            updated.entries.push({
                name: `Preset ${nextPresetNumber}`,
                actions: currentStateActions()
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
        class="contents min-[37.5rem]:order-0 min-[37.5rem]:col-start-2 min-[37.5rem]:row-start-3 min-[37.5rem]:grid min-[37.5rem]:content-start min-[37.5rem]:gap-3 min-[56.25rem]:gap-3.5">
        <section class="order-3 min-w-0 min-[37.5rem]:order-0" aria-label="Global transposition">
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
        <section class="order-5 min-w-0 min-[37.5rem]:order-0" aria-label="Global gain">
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
            class="border-border bg-surface-card order-1 overflow-hidden rounded-xl border min-[37.5rem]:order-0 min-[37.5rem]:col-span-2 min-[37.5rem]:row-start-1">
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
                        aria-label="Create preset from current state"
                        title="Create preset from current state"
                        disabled={loading ||
                            creatingPreset ||
                            activatingPreset !== null ||
                            togglingInstrument !== null}
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
                            class="bg-surface-item border-border-strong hover:border-accent/30 relative grid min-h-18.75 cursor-pointer grid-cols-[1.375rem_minmax(0,1fr)] content-center items-start gap-1.75 rounded-lg border px-2.25 py-2.75 text-left transition-colors disabled:cursor-wait"
                            type="button"
                            disabled={activatingPreset !== null ||
                                togglingInstrument !== null ||
                                creatingPreset}
                            onclick={() => void applyPreset(index)}>
                            {#if activationAnimationPreset === index}
                                <span
                                    use:activationGlow
                                    class="border-accent bg-accent/12 pointer-events-none absolute -inset-px z-10 rounded-lg border-2"
                                    aria-hidden="true">
                                </span>
                            {/if}
                            <small
                                class="text-foreground-index font-mono text-xs min-[56.25rem]:text-sm">
                                {String(index + 1).padStart(2, '0')}
                            </small>
                            <span class="min-w-0">
                                <strong
                                    class={`block overflow-hidden text-sm text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm ${preset.name.trim() ? 'text-foreground' : 'text-foreground-muted italic'}`}>
                                    {presetName(index, preset.name)}
                                </strong>
                                <em
                                    class="text-foreground-muted mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap not-italic min-[56.25rem]:text-sm">
                                    {preset.actions.length}
                                    {preset.actions.length === 1 ? 'action' : 'actions'}
                                </em>
                            </span>
                            {#if activatingPreset === index}
                                <Icon
                                    icon="lucide:rotate-cw"
                                    width="0.9375rem"
                                    class="text-accent absolute right-2 bottom-2 animate-spin" />
                            {/if}
                        </button>
                    {/each}
                </div>
            {/if}
        </section>

        <ProjectTransport
            class="order-2 min-[37.5rem]:order-0 min-[37.5rem]:col-span-2 min-[37.5rem]:row-start-2" />

        <section
            class="border-border bg-surface-card order-4 overflow-hidden rounded-xl border min-[37.5rem]:order-0 min-[37.5rem]:col-start-1 min-[37.5rem]:row-start-3">
            <header
                class="border-border flex min-h-13.5 items-center justify-between border-b px-3.25 min-[56.25rem]:min-h-14.75 min-[56.25rem]:px-3.75">
                <div>
                    <span class="block text-sm font-bold">Instruments</span>
                </div>
                <b
                    class="bg-surface-badge text-foreground-strong grid h-5.75 min-w-0! place-items-center rounded-md px-1.5 text-center font-sans! text-xs leading-none font-semibold tabular-nums min-[56.25rem]:text-sm">
                    <span>{enabledControllerCount} / {instruments.length}</span>
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
                        {@const assignedSynth = synthEntry(instrument.instrument_id)}
                        {@const programName = soundFontProgramName(assignedSynth)}
                        <button
                            class={`grid min-h-14.5 cursor-pointer grid-cols-[0.1875rem_minmax(0,1fr)_2.125rem] items-center gap-2.5 border-0 px-2.5 py-1.75 text-left transition-[filter] enabled:hover:brightness-110 disabled:cursor-wait min-[56.25rem]:min-h-15.5 min-[56.25rem]:px-3.25 ${instrument.controller.data.enabled ? 'bg-surface-item-active' : 'bg-surface-disabled'}`}
                            type="button"
                            aria-pressed={instrument.controller.data.enabled}
                            disabled={togglingInstrument === instrument.id}
                            onclick={() =>
                                void toggleInstrument(
                                    instrument.id,
                                    !instrument.controller.data.enabled
                                )}>
                            <i
                                class={`h-7.25 rounded ${instrument.controller.data.enabled ? '' : 'opacity-35 grayscale'}`}
                                style:background={synthColor(instrument.instrument_id)}>
                            </i>
                            <span class="min-w-0">
                                <strong
                                    class={`block overflow-hidden text-sm font-semibold text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm ${instrument.controller.data.enabled ? 'text-foreground' : 'text-foreground-disabled'}`}>
                                    {controllerName(instrument)}
                                </strong>
                                <small
                                    class="text-foreground-caption mt-1 block overflow-hidden text-xs text-ellipsis whitespace-nowrap min-[56.25rem]:text-sm">
                                    {synthName(assignedSynth)}
                                    {#if programName}
                                        <span class="text-foreground-muted">· {programName}</span>
                                    {/if}
                                </small>
                            </span>
                            <b
                                class={`grid h-8 place-items-center rounded-lg border ${instrument.controller.data.enabled ? 'border-accent/30 bg-accent/10 text-accent' : 'border-border-strong text-foreground-off'}`}>
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

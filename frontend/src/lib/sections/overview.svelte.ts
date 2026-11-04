import { onDestroy, onMount } from 'svelte';
import { decibelsToLinearGain, linearGainToDecibels } from '$lib/gain';
import type { UserPresetAction } from '$lib/generated/models';
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

const ACTIVATION_GLOW_DURATION = 320;

export class OverviewSection {
    loading = $state(true);
    togglingInstrument = $state<number | null>(null);
    activatingPreset = $state<number | null>(null);
    activationAnimationPreset = $state<number | null>(null);
    creatingPreset = $state(false);
    private activationAnimationTimer: ReturnType<typeof setTimeout> | undefined;

    instruments = $derived(controllerEntries().sort((left, right) => left.id - right.id));
    project = $derived(activeProject());
    globalGainDb = $derived(linearGainToDecibels(this.project?.instrument_graph.global_gain ?? 1));
    availableSynths = $derived(synths());
    enabledControllerCount = $derived(
        this.instruments.filter(({ controller }) => controller.data.enabled).length
    );
    presets = $derived(userPresets());
    globalTransposition = $derived(instrumentGlobalTransposition());

    constructor() {
        onDestroy(() => clearTimeout(this.activationAnimationTimer));
        onMount(async () => {
            try {
                await Promise.all([fetchInstruments(), fetchSynths(), fetchUserPresets()]);
            } catch (error) {
                notify.error(error instanceof Error ? error.message : String(error), {
                    title: 'Load live controls'
                });
            } finally {
                this.loading = false;
            }
        });
    }

    activationGlow(node: HTMLSpanElement) {
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
        return { destroy: () => animation.cancel() };
    }

    synthEntry(id: number | null) {
        if (id === null) return undefined;
        return this.availableSynths.find((entry) => entry.id === id);
    }

    synthName(entry: SynthEntry | undefined) {
        return (
            entry?.synth.name.trim() ||
            (entry ? `${entry.kind} instrument #${entry.id + 1}` : 'Instrument')
        );
    }

    soundFontProgramName(entry: SynthEntry | undefined) {
        if (entry?.kind !== SynthKind.Fluid) return undefined;
        const program = entry.synth.program;
        const name = entry.synth.info.banks
            .find((bank) => bank.number === program.bank)
            ?.presets.find((preset) => preset.number === program.preset_id)
            ?.name.trim();
        return name || `Bank ${program.bank} / Program ${program.preset_id}`;
    }

    controllerName(entry: (typeof this.instruments)[number]) {
        const type = entry.controller.type === 'MidiInput' ? 'MIDI input' : 'Drum machine';
        const name = entry.controller.data.name.trim();
        return name ? `${type}: ${name}` : type;
    }

    presetName(index: number, name: string) {
        return name.trim() || `Unnamed preset #${index + 1}`;
    }

    private currentStateActions(): UserPresetAction[] {
        return [
            'DisableAllInstrumentControllers',
            ...this.instruments
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
            ...this.availableSynths.flatMap((entry) =>
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

    synthColor(id: number | null) {
        switch (this.synthEntry(id)?.kind) {
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

    async setGlobalTransposition(value: number) {
        await setInstrumentGlobalTransposition(value);
        return value;
    }

    async toggleInstrument(id: number, enabled: boolean) {
        if (
            this.togglingInstrument !== null ||
            this.activatingPreset !== null ||
            this.creatingPreset
        )
            return;
        this.togglingInstrument = id;
        try {
            await setControllerEnabled(id, enabled);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: enabled ? 'Enable instrument' : 'Disable instrument'
            });
        } finally {
            this.togglingInstrument = null;
        }
    }

    async applyPreset(index: number) {
        if (
            this.activatingPreset !== null ||
            this.togglingInstrument !== null ||
            this.creatingPreset
        )
            return;
        clearTimeout(this.activationAnimationTimer);
        this.activationAnimationPreset = null;
        this.activatingPreset = index;
        try {
            await activateUserPreset(index);
            this.activationAnimationPreset = index;
            this.activationAnimationTimer = setTimeout(() => {
                this.activationAnimationPreset = null;
            }, ACTIVATION_GLOW_DURATION + 30);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: `Activate ${userPresets().entries[index]?.name ?? 'preset'}`
            });
        } finally {
            this.activatingPreset = null;
        }
    }

    async createPreset() {
        if (
            this.creatingPreset ||
            this.activatingPreset !== null ||
            this.togglingInstrument !== null
        )
            return;
        this.creatingPreset = true;
        const nextPresetNumber = userPresets().entries.length + 1;
        try {
            const updated = cloneUserPresets(userPresets());
            updated.entries.push({
                name: `Preset ${nextPresetNumber}`,
                actions: this.currentStateActions()
            });
            await saveUserPresets(updated);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: 'Create user preset'
            });
        } finally {
            this.creatingPreset = false;
        }
    }

    async setGlobalGainDb(value: number) {
        await setGlobalGain(decibelsToLinearGain(value));
        return value;
    }
}

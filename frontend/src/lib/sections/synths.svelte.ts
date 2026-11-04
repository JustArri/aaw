import { onMount } from 'svelte';
import { decibelsToLinearGain, linearGainToDecibels } from '$lib/gain';
import { notify } from '$lib/push-notifications.svelte';
import { activeProject } from '$lib/state/active-project.svelte';
import { assetsForSynth, clapPlugins, fetchAssets, refreshAssets } from '$lib/state/assets.svelte';
import { setGlobalEffects, setSynthEffects } from '$lib/state/effects.svelte';
import { fetchPreferences, preferences } from '$lib/state/system.svelte';
import {
    addSynth,
    fetchSynths,
    instrumentSlots,
    removeSynth,
    replaceSynth,
    setClapParameter,
    setFluidProgram,
    setSynthGain,
    setSynthName,
    synths,
    SynthKind,
    type SynthKind as SynthKindType,
    type SynthEntry
} from '$lib/state/synths.svelte';
import { unreachable } from '$lib/util';
import type { Effects, InstrumentPushRequest as SynthPushParams } from '$lib/generated/models';
import { errorToUserMessage } from '$lib/error';
import { synthColors } from '$lib/content/synths';

export { SynthKind };

export class SynthsSection {
    readonly colors = synthColors;
    filter = $state<'All' | SynthKindType>('All');
    selectedKey = $state('');
    inspectorOpen = $state(false);
    addOpen = $state(false);
    addKind = $state<SynthKindType>(SynthKind.Fluid);
    addPath = $state('');
    addPluginId = $state('');
    addMaxVoices = $state(256);
    addPreloadSize = $state(65_536);
    editName = $state('');
    replacementKind = $state<SynthKindType>(SynthKind.Fluid);
    replacementPath = $state('');
    replacementPluginId = $state('');
    replacementMaxVoices = $state(256);
    replacementPreloadSize = $state(65_536);
    selectedProgram = $state('');
    busy = $state(false);
    busyAction = $state<string | null>(null);
    nameSaving = $state(false);
    programBusy = $state(false);
    removing = $state(false);
    loading = $state(true);
    reloadingSynths = $state(false);
    synthsReloaded = $state(false);
    clapParameterBusy = $state(new Set<number>());

    allSynths = $derived(synths());
    project = $derived(activeProject());
    visibleSynths = $derived(
        this.filter === 'All'
            ? this.allSynths
            : this.allSynths.filter((entry) => entry.kind === this.filter)
    );
    selected = $derived(
        this.allSynths.find((entry) => this.key(entry) === this.selectedKey) ?? this.allSynths[0]
    );
    selectedName = $derived(this.selected?.synth.name ?? '');
    nameChanged = $derived(
        Boolean(this.selected && this.editName.trim() !== this.selected.synth.name)
    );
    references = $derived(
        this.selected
            ? Object.entries(instrumentSlots()).filter(
                  ([, slot]) => slot.synth_id === this.selected?.id
              )
            : []
    );
    visibleClapParameters = $derived(
        this.selected?.kind === SynthKind.Clap
            ? this.selected.synth.plugin_parameters.filter(
                  (parameter) => (parameter.flags & 4) === 0
              )
            : []
    );
    programSuggestions = $derived(
        this.selected?.kind === SynthKind.Fluid
            ? this.selected.synth.info.banks.flatMap((bank) =>
                  bank.presets.map((preset) => ({
                      value: `${bank.number}:${preset.number}`,
                      label: preset.name,
                      selectionLabel: preset.name,
                      description: `Bank ${bank.number} · Program ${preset.number}`
                  }))
              )
            : []
    );
    readonlyParameterFlag = 1 << 3;

    constructor() {
        onMount(async () => {
            try {
                await Promise.all([fetchSynths(), fetchAssets(), fetchPreferences()]);
                this.addMaxVoices = this.suggestedMaxVoices(this.addKind);
                this.addPreloadSize = this.suggestedPreloadSize();
                const requestedSynth = new URL(window.location.href).searchParams.get('instrument');
                const initial =
                    (requestedSynth === null
                        ? undefined
                        : synths().find((entry) => entry.id === Number(requestedSynth))) ??
                    synths()[0];
                if (initial) this.select(initial, requestedSynth !== null);
            } catch (cause) {
                notify.error(errorToUserMessage(cause));
            } finally {
                this.loading = false;
            }
        });
        $effect(() => {
            if (this.programBusy || this.selected?.kind !== SynthKind.Fluid) return;
            this.selectedProgram = `${this.selected.synth.program.bank}:${this.selected.synth.program.preset_id}`;
        });
        $effect(() => {
            this.editName = this.selectedName;
        });
    }

    clapPlugins() {
        return clapPlugins();
    }
    assetsForSynth(kind: SynthKindType) {
        return assetsForSynth(kind);
    }
    refreshAssets = refreshAssets;

    async saveGlobalEffects(effects: Effects) {
        return setGlobalEffects(effects);
    }
    async saveSynthEffects(id: number, effects: Effects) {
        return setSynthEffects(id, effects);
    }
    async setGain(id: number, valueDb: number) {
        await setSynthGain(id, decibelsToLinearGain(valueDb));
        return valueDb;
    }
    gainDb(value: number) {
        return linearGainToDecibels(value);
    }

    async submitClapParameter(parameterId: number, value: number) {
        if (!this.selected || this.selected.kind !== SynthKind.Clap) return;
        this.clapParameterBusy = new Set(this.clapParameterBusy).add(parameterId);
        try {
            await setClapParameter(this.selected.id, parameterId, value);
        } catch (error) {
            notify.error(errorToUserMessage(error), { title: 'Set plugin parameter' });
        } finally {
            const next = new Set(this.clapParameterBusy);
            next.delete(parameterId);
            this.clapParameterBusy = next;
        }
    }

    key(entry: SynthEntry) {
        return String(entry.id);
    }
    displayName(entry: SynthEntry) {
        return entry.synth.name.trim() || `${entry.kind} instrument #${entry.id + 1}`;
    }
    instrumentType(kind: SynthKindType) {
        switch (kind) {
            case 'Dummy':
                return 'Dummy Synth';
            case 'Fluid':
                return 'Fluid Synth';
            case 'Sfizz':
                return 'Sfizz Synth';
            case 'Clap':
                return 'CLAP Plugin';
            default:
                unreachable(kind);
        }
    }
    suggestedMaxVoices(kind: SynthKindType) {
        const value = preferences();
        switch (kind) {
            case 'Dummy':
                return value?.dummy_max_voices ?? 32;
            case 'Fluid':
                return value?.fluid_max_voices ?? 256;
            case 'Sfizz':
                return value?.sfizz_max_voices ?? 64;
            case 'Clap':
                return 1;
            default:
                unreachable(kind);
        }
    }
    suggestedPreloadSize() {
        return preferences()?.sfizz_preload_size ?? 65_536;
    }

    select(entry: SynthEntry, open = true) {
        this.selectedKey = this.key(entry);
        this.editName = entry.synth.name;
        this.replacementKind = entry.kind;
        this.replacementPath = entry.kind === SynthKind.Dummy ? '' : entry.synth.file_path;
        this.replacementPluginId = entry.kind === SynthKind.Clap ? entry.synth.plugin_id : '';
        this.replacementMaxVoices = entry.kind === SynthKind.Clap ? 1 : entry.synth.max_voices;
        this.replacementPreloadSize =
            entry.kind === SynthKind.Sfizz ? entry.synth.preload_size : this.suggestedPreloadSize();
        this.selectedProgram =
            entry.kind === SynthKind.Fluid
                ? `${entry.synth.program.bank}:${entry.synth.program.preset_id}`
                : '';
        this.inspectorOpen = open;
    }

    toggleAdd() {
        this.addOpen = !this.addOpen;
        if (this.addOpen && this.filter !== 'All') {
            this.addKind = this.filter;
            this.addMaxVoices = this.suggestedMaxVoices(this.addKind);
            this.addPreloadSize = this.suggestedPreloadSize();
        }
    }

    private async run(operation: () => Promise<unknown>, title: string) {
        if (this.busy) return;
        this.busy = true;
        this.busyAction = title;
        try {
            return await operation();
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title });
        } finally {
            this.busy = false;
            this.busyAction = null;
        }
    }

    presetName(entry: SynthEntry) {
        if (entry.kind !== SynthKind.Fluid) return '';
        const bank = entry.synth.info.banks.find(
            (item) => item.number === entry.synth.program.bank
        );
        return (
            bank?.presets.find((item) => item.number === entry.synth.program.preset_id)?.name ??
            'Using fallback'
        );
    }
    hasSelectedProgram(entry: SynthEntry) {
        if (entry.kind !== SynthKind.Fluid) return false;
        return entry.synth.info.banks
            .find((bank) => bank.number === entry.synth.program.bank)
            ?.presets.some((preset) => preset.number === entry.synth.program.preset_id);
    }
    details(entry: SynthEntry) {
        if (entry.kind === SynthKind.Fluid) {
            const presets = entry.synth.info.banks.reduce(
                (total, bank) => total + bank.presets.length,
                0
            );
            return `${entry.synth.info.banks.length} banks · ${presets} presets · ${entry.synth.max_voices} voices`;
        }
        if (entry.kind === SynthKind.Sfizz)
            return `SFZ instrument · ${entry.synth.max_voices} voices · ${entry.synth.preload_size.toLocaleString()} preload frames`;
        if (entry.kind === SynthKind.Clap) {
            const parameterCount = entry.synth.plugin_parameters.filter(
                (parameter) => (parameter.flags & 4) === 0
            ).length;
            return [
                entry.synth.plugin_name || entry.synth.plugin_id,
                entry.synth.plugin_version,
                `${parameterCount} parameters`
            ]
                .filter(Boolean)
                .join(' · ');
        }
        return `Diagnostic synth · ${entry.synth.max_voices} voices`;
    }
    formatParameterNumber(value: number) {
        return Number.isInteger(value)
            ? String(value)
            : value.toLocaleString(undefined, { maximumFractionDigits: 4 });
    }
    parameterValue(parameter: { value: number | null; value_text: string | null }) {
        const text = parameter.value_text?.trim();
        if (text) return text;
        return parameter.value === null
            ? 'Unavailable'
            : this.formatParameterNumber(parameter.value);
    }

    async submitAdd() {
        let createdId: number | undefined;
        await this.run(async () => {
            createdId = await addSynth(
                this.createDetails(
                    this.addKind,
                    this.addMaxVoices,
                    this.addPreloadSize,
                    this.addPath,
                    this.addPluginId
                )
            );
            return createdId;
        }, 'Create instrument');
        if (typeof createdId === 'number') {
            const added = synths().find(
                (entry) => entry.kind === this.addKind && entry.id === createdId
            );
            if (added) this.select(added);
            this.addOpen = false;
            this.addPath = '';
            this.addPluginId = '';
        }
    }

    async submitName() {
        if (!this.selected || !this.editName.trim() || !this.nameChanged || this.nameSaving) return;
        this.nameSaving = true;
        const name = this.editName.trim();
        try {
            await this.run(() => setSynthName(this.selected!.id, name), 'Rename instrument');
            this.editName = name;
        } finally {
            this.nameSaving = false;
        }
    }

    async submitReplacement() {
        if (
            !this.selected ||
            (this.replacementKind !== SynthKind.Dummy && !this.replacementPath.trim()) ||
            (this.replacementKind === SynthKind.Clap && !this.replacementPluginId)
        )
            return;
        const id = this.selected.id;
        let succeeded = false;
        await this.run(async () => {
            await replaceSynth(
                id,
                this.createDetails(
                    this.replacementKind,
                    this.replacementMaxVoices,
                    this.replacementPreloadSize,
                    this.replacementPath.trim(),
                    this.replacementPluginId
                )
            );
            succeeded = true;
        }, 'Replace instrument');
        if (succeeded) {
            const replaced = synths().find((entry) => entry.id === id);
            if (replaced) this.select(replaced);
        }
    }

    async submitProgram() {
        if (this.selected?.kind !== SynthKind.Fluid || this.busy || this.programBusy) return;
        const previousProgram = `${this.selected.synth.program.bank}:${this.selected.synth.program.preset_id}`;
        const [bank, presetId] = this.selectedProgram.split(':').map(Number);
        this.programBusy = true;
        try {
            await setFluidProgram(this.selected.id, { bank, preset_id: presetId });
        } catch (cause) {
            this.selectedProgram = previousProgram;
            notify.error(errorToUserMessage(cause), { title: 'Select program' });
        } finally {
            this.programBusy = false;
        }
    }

    async stepProgram(direction: -1 | 1) {
        if (this.busy || this.programBusy || this.selected?.kind !== SynthKind.Fluid) return;
        const programs = this.selected.synth.info.banks.flatMap((bank) =>
            bank.presets.map((preset) => `${bank.number}:${preset.number}`)
        );
        const currentIndex = programs.indexOf(this.selectedProgram);
        const nextIndex = Math.max(0, Math.min(programs.length - 1, currentIndex + direction));
        if (nextIndex === currentIndex || programs[nextIndex] === undefined) return;
        this.selectedProgram = programs[nextIndex];
        await this.submitProgram();
    }

    handleProgramKeydown(event: KeyboardEvent) {
        if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
        const target = event.target;
        if (
            target instanceof HTMLElement &&
            target.id !== 'synth-program' &&
            (target.matches('input, textarea, select') || target.isContentEditable)
        )
            return;
        event.preventDefault();
        void this.stepProgram(event.key === 'ArrowLeft' ? -1 : 1);
    }

    async submitRemove() {
        if (!this.selected) return;
        const removedKey = this.selectedKey;
        this.removing = true;
        await this.run(() => removeSynth(this.selected!.id), 'Remove synth');
        this.removing = false;
        if (!synths().some((entry) => this.key(entry) === removedKey)) {
            this.selectedKey = '';
            this.inspectorOpen = false;
            const first = synths()[0];
            if (first) this.select(first, false);
        }
    }

    async reloadSynths() {
        if (this.reloadingSynths) return;
        this.reloadingSynths = true;
        this.synthsReloaded = false;
        try {
            await fetchSynths();
            this.synthsReloaded = true;
            setTimeout(() => (this.synthsReloaded = false), 1200);
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Reload instruments' });
        } finally {
            this.reloadingSynths = false;
        }
    }

    private createDetails(
        type: SynthKind,
        maxVoices: number,
        preloadSize: number,
        path: string,
        pluginId: string
    ): SynthPushParams {
        switch (type) {
            case 'Dummy':
                return { type, data: { max_voices: maxVoices } };
            case 'Fluid':
                return { type, data: { max_voices: maxVoices, path } };
            case 'Sfizz':
                return {
                    type,
                    data: { max_voices: maxVoices, preload_size: preloadSize, path }
                };
            case 'Clap':
                return { type, data: { path, plugin_id: pluginId } };
            default:
                unreachable(type);
        }
    }

    assetPlaceholder(type: SynthKind): string | undefined {
        switch (type) {
            case 'Dummy':
                return undefined;
            case 'Fluid':
                return 'fluid/soundfont.sf2';
            case 'Sfizz':
                return 'sfizz/instrument.sfz';
            case 'Clap':
                return undefined;
            default:
                unreachable(type);
        }
    }
}

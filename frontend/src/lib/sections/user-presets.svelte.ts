import { onMount } from 'svelte';
import type { ParameterInfo, UserPresetAction } from '$lib/generated/models';
import { enabledOptions, userPresetActionTypeOptions } from '$lib/content/user-presets';
import { notify } from '$lib/push-notifications.svelte';
import { controllerEntries, fetchSynths, synths, SynthKind } from '$lib/state/synths.svelte';
import {
    activateUserPreset,
    cloneUserPresets,
    fetchUserPresets,
    initializeUserPresetsDraft,
    resetUserPresetsDraft,
    saveUserPresets,
    userPresets,
    userPresetsDraft,
    type EditableUserPreset
} from '$lib/state/user-presets.svelte';

export class UserPresetsSection {
    readonly draft = userPresetsDraft;
    readonly actionTypeOptions = userPresetActionTypeOptions;
    readonly enabledOptions = enabledOptions;
    loading = $state(true);
    busy = $state(false);
    activating = $state<number | null>(null);
    draggedPreset = $state<EditableUserPreset | null>(null);
    dragX = $state(0);
    dragY = $state(0);
    dropInsertionIndex = $state<number | null>(null);
    instruments = $derived(synths().sort((left, right) => left.id - right.id));
    controllers = $derived(
        controllerEntries().sort(
            (left, right) =>
                left.instrument_id - right.instrument_id || left.controller_id - right.controller_id
        )
    );
    fluidInstruments = $derived(
        this.instruments.filter((instrument) => instrument.kind === SynthKind.Fluid)
    );
    clapInstruments = $derived(
        this.instruments.filter((instrument) => instrument.kind === SynthKind.Clap)
    );
    changed = $derived(JSON.stringify(userPresetsDraft) !== JSON.stringify(userPresets()));

    constructor() {
        onMount(async () => {
            try {
                const [presets] = await Promise.all([fetchUserPresets(), fetchSynths()]);
                initializeUserPresetsDraft(presets);
            } catch (error) {
                notify.error(error instanceof Error ? error.message : String(error), {
                    title: 'Load user presets'
                });
            } finally {
                this.loading = false;
            }
        });
    }

    addPreset() {
        this.draft.entries.push({
            name: `Preset ${this.draft.entries.length + 1}`,
            actions: ['DisableAllInstrumentControllers']
        });
    }
    removePreset(index: number) {
        this.draft.entries.splice(index, 1);
    }
    movePreset(index: number, offset: number) {
        const destination = index + offset;
        if (destination < 0 || destination >= this.draft.entries.length) return;
        const [preset] = this.draft.entries.splice(index, 1);
        this.draft.entries.splice(destination, 0, preset);
    }
    startPresetDrag(event: PointerEvent, preset: EditableUserPreset) {
        if (event.button !== 0) return;
        const handle = event.currentTarget as HTMLElement;
        handle.setPointerCapture(event.pointerId);
        event.preventDefault();
        this.draggedPreset = preset;
        this.dragX = event.clientX;
        this.dragY = event.clientY;
        this.dropInsertionIndex = this.draft.entries.indexOf(preset);
    }
    dragPreset(event: PointerEvent) {
        if (!this.draggedPreset) return;
        this.dragX = event.clientX;
        this.dragY = event.clientY;
        const target = document
            .elementFromPoint(event.clientX, event.clientY)
            ?.closest<HTMLElement>('[data-preset-index]');
        const targetIndex = Number(target?.dataset.presetIndex);
        if (!target || !Number.isInteger(targetIndex)) return;
        const bounds = target.getBoundingClientRect();
        this.dropInsertionIndex =
            targetIndex + (event.clientY > bounds.top + bounds.height / 2 ? 1 : 0);
    }
    stopPresetDrag(event: PointerEvent) {
        const handle = event.currentTarget as HTMLElement;
        if (handle.hasPointerCapture(event.pointerId))
            handle.releasePointerCapture(event.pointerId);
        if (this.draggedPreset && this.dropInsertionIndex !== null) {
            const source = this.draft.entries.indexOf(this.draggedPreset);
            let destination = this.dropInsertionIndex;
            if (destination > source) destination -= 1;
            if (source >= 0 && destination !== source) {
                const [preset] = this.draft.entries.splice(source, 1);
                this.draft.entries.splice(destination, 0, preset);
            }
        }
        this.draggedPreset = null;
        this.dropInsertionIndex = null;
    }
    cancelPresetDrag(event: PointerEvent) {
        const handle = event.currentTarget as HTMLElement;
        if (handle.hasPointerCapture(event.pointerId))
            handle.releasePointerCapture(event.pointerId);
        this.draggedPreset = null;
        this.dropInsertionIndex = null;
    }

    actionType(action: UserPresetAction) {
        if (action === 'DisableAllInstrumentControllers') return 'disable-all';
        if ('SetInstrumentControllerEnabled' in action) return 'set-controller';
        return 'SetSoundFontProgram' in action ? 'set-program' : 'set-clap-parameter';
    }
    controllerKey(instrumentId: number, controllerId: number) {
        return String(instrumentId * 65_536 + controllerId);
    }
    private controllerName(entry: (typeof this.controllers)[number]) {
        const instrument = this.instruments.find(
            (candidate) => candidate.id === entry.instrument_id
        );
        const instrumentName =
            instrument?.synth.name.trim() || `Instrument #${entry.instrument_id + 1}`;
        const type = entry.controller.type === 'MidiInput' ? 'MIDI input' : 'Drum machine';
        const name = entry.controller.data.name.trim();
        return `${instrumentName} · ${name ? `${type}: ${name}` : type}`;
    }
    controllerOptions(action: UserPresetAction) {
        const options = this.controllers.map((entry) => ({
            value: this.controllerKey(entry.instrument_id, entry.controller_id),
            label: this.controllerName(entry)
        }));
        if (
            action === 'DisableAllInstrumentControllers' ||
            !('SetInstrumentControllerEnabled' in action)
        )
            return options;
        const address = action.SetInstrumentControllerEnabled.controller;
        const value = this.controllerKey(address.instrument_id, address.controller_id);
        return options.some((option) => option.value === value)
            ? options
            : [
                  {
                      value,
                      label: `Missing controller ${address.instrument_id}:${address.controller_id}`
                  },
                  ...options
              ];
    }
    instrumentOptions(action: UserPresetAction) {
        const options = this.fluidInstruments.map((instrument) => ({
            value: String(instrument.id),
            label: instrument.synth.name.trim() || `SoundFont instrument #${instrument.id + 1}`
        }));
        if (action === 'DisableAllInstrumentControllers' || !('SetSoundFontProgram' in action))
            return options;
        const value = String(action.SetSoundFontProgram.instrument);
        return options.some((option) => option.value === value)
            ? options
            : [{ value, label: `Missing instrument #${Number(value) + 1}` }, ...options];
    }
    programOptions(action: UserPresetAction) {
        if (action === 'DisableAllInstrumentControllers' || !('SetSoundFontProgram' in action))
            return [];
        const data = action.SetSoundFontProgram;
        const instrument = this.fluidInstruments.find(
            (candidate) => candidate.id === data.instrument
        );
        const options =
            instrument?.synth.info.banks.flatMap((bank) =>
                bank.presets.map((program) => ({
                    value: `${bank.number}:${program.number}`,
                    label: program.name,
                    description: `Bank ${bank.number} · Program ${program.number}`
                }))
            ) ?? [];
        const value = `${data.program.bank}:${data.program.preset_id}`;
        return options.some((option) => option.value === value)
            ? options
            : [
                  { value, label: `Bank ${data.program.bank} · Program ${data.program.preset_id}` },
                  ...options
              ];
    }
    private writableClapParameters(instrument: (typeof this.clapInstruments)[number] | undefined) {
        return (
            instrument?.synth.plugin_parameters.filter(
                (parameter) => (parameter.flags & 4) === 0 && (parameter.flags & 8) === 0
            ) ?? []
        );
    }
    clapInstrumentOptions(action: UserPresetAction) {
        const options = this.clapInstruments.map((instrument) => ({
            value: String(instrument.id),
            label: instrument.synth.name.trim() || `CLAP instrument #${instrument.id + 1}`
        }));
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action))
            return options;
        const value = String(action.SetClapParameter.instrument);
        return options.some((option) => option.value === value)
            ? options
            : [{ value, label: `Missing CLAP instrument #${Number(value) + 1}` }, ...options];
    }
    clapParameter(action: UserPresetAction): ParameterInfo | undefined {
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action))
            return undefined;
        const data = action.SetClapParameter;
        return this.clapInstruments
            .find((instrument) => instrument.id === data.instrument)
            ?.synth.plugin_parameters.find((parameter) => parameter.id === data.parameter_id);
    }
    clapParameterOptions(action: UserPresetAction) {
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action))
            return [];
        const data = action.SetClapParameter;
        const instrument = this.clapInstruments.find(
            (candidate) => candidate.id === data.instrument
        );
        const options = this.writableClapParameters(instrument).map((parameter) => ({
            value: String(parameter.id),
            label: parameter.name || `Parameter ${parameter.id}`,
            description: parameter.module || undefined
        }));
        const value = String(data.parameter_id);
        return options.some((option) => option.value === value)
            ? options
            : [{ value, label: `Missing parameter ${value}` }, ...options];
    }
    parameterPrecision(parameter: ParameterInfo | undefined) {
        return parameter && (parameter.flags & 1) !== 0 ? 0 : 3;
    }
    parameterSmallStep(parameter: ParameterInfo | undefined) {
        if (!parameter) return 0.01;
        return (parameter.flags & 1) !== 0
            ? 1
            : Math.max((parameter.max_value - parameter.min_value) / 100, 0.001);
    }

    private replaceAction(presetIndex: number, actionIndex: number, action: UserPresetAction) {
        this.draft.entries[presetIndex].actions[actionIndex] = action;
    }
    setActionType(presetIndex: number, actionIndex: number, type: string) {
        if (type === 'disable-all') {
            this.replaceAction(presetIndex, actionIndex, 'DisableAllInstrumentControllers');
            return;
        }
        if (type === 'set-controller') {
            const controller = this.controllers[0];
            this.replaceAction(presetIndex, actionIndex, {
                SetInstrumentControllerEnabled: {
                    controller: {
                        instrument_id: controller?.instrument_id ?? 0,
                        controller_id: controller?.controller_id ?? 0
                    },
                    flag: true
                }
            });
            return;
        }
        if (type === 'set-program') {
            const instrument = this.fluidInstruments[0];
            this.replaceAction(presetIndex, actionIndex, {
                SetSoundFontProgram: {
                    instrument: instrument?.id ?? 0,
                    program: { ...(instrument?.synth.program ?? { bank: 0, preset_id: 0 }) }
                }
            });
            return;
        }
        const instrument = this.clapInstruments[0];
        const parameter = this.writableClapParameters(instrument)[0];
        this.replaceAction(presetIndex, actionIndex, {
            SetClapParameter: {
                instrument: instrument?.id ?? 0,
                parameter_id: parameter?.id ?? 0,
                value: parameter?.value ?? parameter?.default_value ?? 0
            }
        });
    }
    setActionController(presetIndex: number, actionIndex: number, value: string) {
        const action = this.draft.entries[presetIndex].actions[actionIndex];
        if (
            action === 'DisableAllInstrumentControllers' ||
            !('SetInstrumentControllerEnabled' in action)
        )
            return;
        this.replaceAction(presetIndex, actionIndex, {
            SetInstrumentControllerEnabled: {
                ...action.SetInstrumentControllerEnabled,
                controller: {
                    instrument_id: Math.floor(Number(value) / 65_536),
                    controller_id: Number(value) % 65_536
                }
            }
        });
    }
    setActionEnabled(presetIndex: number, actionIndex: number, value: string) {
        const action = this.draft.entries[presetIndex].actions[actionIndex];
        if (
            action === 'DisableAllInstrumentControllers' ||
            !('SetInstrumentControllerEnabled' in action)
        )
            return;
        this.replaceAction(presetIndex, actionIndex, {
            SetInstrumentControllerEnabled: {
                ...action.SetInstrumentControllerEnabled,
                flag: value === 'true'
            }
        });
    }
    setActionInstrument(presetIndex: number, actionIndex: number, value: string) {
        const instrument = this.fluidInstruments.find(
            (candidate) => candidate.id === Number(value)
        );
        this.replaceAction(presetIndex, actionIndex, {
            SetSoundFontProgram: {
                instrument: Number(value),
                program: { ...(instrument?.synth.program ?? { bank: 0, preset_id: 0 }) }
            }
        });
    }
    setActionProgram(presetIndex: number, actionIndex: number, value: string) {
        const action = this.draft.entries[presetIndex].actions[actionIndex];
        if (action === 'DisableAllInstrumentControllers' || !('SetSoundFontProgram' in action))
            return;
        const [bank, presetId] = value.split(':').map(Number);
        this.replaceAction(presetIndex, actionIndex, {
            SetSoundFontProgram: {
                ...action.SetSoundFontProgram,
                program: { bank, preset_id: presetId }
            }
        });
    }
    setActionClapInstrument(presetIndex: number, actionIndex: number, value: string) {
        const instrument = this.clapInstruments.find((candidate) => candidate.id === Number(value));
        const parameter = this.writableClapParameters(instrument)[0];
        this.replaceAction(presetIndex, actionIndex, {
            SetClapParameter: {
                instrument: Number(value),
                parameter_id: parameter?.id ?? 0,
                value: parameter?.value ?? parameter?.default_value ?? 0
            }
        });
    }
    setActionClapParameter(presetIndex: number, actionIndex: number, value: string) {
        const action = this.draft.entries[presetIndex].actions[actionIndex];
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action)) return;
        const instrument = this.clapInstruments.find(
            (candidate) => candidate.id === action.SetClapParameter.instrument
        );
        const parameter = this.writableClapParameters(instrument).find(
            (candidate) => candidate.id === Number(value)
        );
        this.replaceAction(presetIndex, actionIndex, {
            SetClapParameter: {
                ...action.SetClapParameter,
                parameter_id: Number(value),
                value: parameter?.value ?? parameter?.default_value ?? 0
            }
        });
    }
    setActionClapValue(presetIndex: number, actionIndex: number, value: number) {
        const action = this.draft.entries[presetIndex].actions[actionIndex];
        if (action === 'DisableAllInstrumentControllers' || !('SetClapParameter' in action)) return;
        this.replaceAction(presetIndex, actionIndex, {
            SetClapParameter: { ...action.SetClapParameter, value }
        });
    }
    addAction(presetIndex: number) {
        this.draft.entries[presetIndex].actions.push('DisableAllInstrumentControllers');
    }
    removeAction(presetIndex: number, actionIndex: number) {
        this.draft.entries[presetIndex].actions.splice(actionIndex, 1);
    }
    moveAction(presetIndex: number, actionIndex: number, offset: number) {
        const actions = this.draft.entries[presetIndex].actions;
        const destination = actionIndex + offset;
        if (destination < 0 || destination >= actions.length) return;
        const [action] = actions.splice(actionIndex, 1);
        actions.splice(destination, 0, action);
    }
    discardChanges() {
        resetUserPresetsDraft();
    }
    async save() {
        if (this.busy || !this.changed) return;
        if (this.draft.entries.some((preset) => !preset.name.trim())) {
            notify.error('Every preset needs a name.', { title: 'Save user presets' });
            return;
        }
        this.busy = true;
        try {
            this.draft.entries = this.draft.entries.map((preset) => ({
                ...preset,
                name: preset.name.trim()
            }));
            await saveUserPresets(cloneUserPresets(this.draft));
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: 'User presets'
            });
        } finally {
            this.busy = false;
        }
    }
    async activate(index: number) {
        if (this.activating !== null || this.changed) return;
        this.activating = index;
        try {
            await activateUserPreset(index);
        } catch (error) {
            notify.error(error instanceof Error ? error.message : String(error), {
                title: this.draft.entries[index].name
            });
        } finally {
            this.activating = null;
        }
    }
}

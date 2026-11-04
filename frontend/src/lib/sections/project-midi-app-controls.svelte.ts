import { onDestroy, onMount } from 'svelte';
import {
    projectMidiControlTypeOptions,
    type ProjectMidiControlType
} from '$lib/content/midi-app-controls';
import { midiChannels, midiSources } from '$lib/content/midi';
import { errorToUserMessage } from '$lib/error';
import {
    MidiFilterKind,
    type CtrMidiProjectControl,
    type CtrMidiProjectControls,
    type MidiFilter
} from '$lib/generated/models';
import { notify } from '$lib/push-notifications.svelte';
import { activeProject } from '$lib/state/active-project.svelte';
import {
    fetchProjectMidiAppControls,
    saveProjectMidiAppControls
} from '$lib/state/midi-app-controls.svelte';
import { subscribeMidiEvents, type MidiEventEntry } from '$lib/state/midi-events.svelte';
import { controllerEntries, fetchSynths, synths } from '$lib/state/synths.svelte';
import { fetchUserPresets, userPresets } from '$lib/state/user-presets.svelte';

export { MidiFilterKind };

export class ProjectMidiAppControlsSection {
    readonly midiSources = midiSources;
    readonly midiChannels = midiChannels;
    drafts = $state<CtrMidiProjectControl[]>([]);
    dirty = $state(new Set<number>());
    loading = $state(true);
    saving = $state(false);
    learningIndex = $state<number | null>(null);
    private initialized = $state(false);
    private unsubscribeMidiEvents: (() => void) | undefined;

    controllerOptions = $derived.by(() => {
        const instrumentNames = new Map(
            synths().map((instrument) => [instrument.id, instrument.synth.name])
        );
        return controllerEntries().map(({ instrument_id, controller_id, controller }) => ({
            value: `${instrument_id}:${controller_id}`,
            label: `${instrumentNames.get(instrument_id) || `Untitled instrument #${instrument_id + 1}`} · ${controller.type === 'MidiInput' ? 'MIDI input' : 'Drum machine'}: ${controller.data.name || `Controller #${controller_id + 1}`}`
        }));
    });
    presetOptions = $derived.by(() =>
        userPresets().entries.map((preset, index) => ({
            value: String(index),
            label: preset.name || `User preset #${index + 1}`
        }))
    );
    controlTypes = $derived(
        projectMidiControlTypeOptions.map(({ availability, ...option }) => ({
            ...option,
            disabled:
                availability === 'controller'
                    ? this.controllerOptions.length === 0
                    : this.presetOptions.length === 0
        }))
    );

    constructor() {
        $effect(() => {
            const controls = activeProject()?.midi_app_controls.controls;
            if (this.initialized && controls && this.dirty.size === 0 && !this.saving) {
                this.drafts = this.snapshotControls(controls);
            }
        });
        onMount(async () => {
            this.unsubscribeMidiEvents = subscribeMidiEvents((entry) =>
                this.handleLearnEvent(entry)
            );
            try {
                const [controls] = await Promise.all([
                    fetchProjectMidiAppControls(),
                    fetchSynths(),
                    fetchUserPresets()
                ]);
                this.drafts = this.snapshotControls(controls?.controls ?? []);
                this.initialized = true;
            } catch (error) {
                notify.error(errorToUserMessage(error), {
                    title: 'Load project MIDI app controls'
                });
            } finally {
                this.loading = false;
            }
        });
        onDestroy(() => this.unsubscribeMidiEvents?.());
    }

    private snapshotControls(controls: CtrMidiProjectControl[]) {
        return $state.snapshot(controls);
    }

    private defaultFilter(): MidiFilter {
        return { type: MidiFilterKind.Denylist, list: [] };
    }

    private firstControllerAddress() {
        const [instrument = '0', controller = '0'] = (
            this.controllerOptions[0]?.value ?? '0:0'
        ).split(':');
        return { instrument: Number(instrument), controller: Number(controller) };
    }

    private defaultControl(): CtrMidiProjectControl {
        const filters = {
            source_filter: this.defaultFilter(),
            channel_filter: this.defaultFilter()
        };
        if (this.controllerOptions.length > 0) {
            return {
                name: '',
                ...filters,
                kind: {
                    NoteInstrumentControllerToggle: {
                        key: 60,
                        ...this.firstControllerAddress()
                    }
                }
            };
        }
        return {
            name: '',
            ...filters,
            kind: { NoteUserPresetSelector: { key: 60, preset_index: 0 } }
        };
    }

    controlType(control: CtrMidiProjectControl): ProjectMidiControlType {
        if ('NoteInstrumentControllerToggle' in control.kind) return 'note-controller-toggle';
        if ('NoteInstrumentControllerEnable' in control.kind) return 'note-controller-enable';
        if ('NoteInstrumentControllerDisable' in control.kind) return 'note-controller-disable';
        if ('CCInstrumentControllerSwitch' in control.kind) return 'cc-controller-switch';
        if ('NoteUserPresetSelector' in control.kind) return 'note-preset';
        return 'cc-preset';
    }

    isControllerControl(control: CtrMidiProjectControl) {
        return !(
            'NoteUserPresetSelector' in control.kind || 'CCUserPresetSelector' in control.kind
        );
    }

    isNoteControl(control: CtrMidiProjectControl) {
        return !(
            'CCInstrumentControllerSwitch' in control.kind || 'CCUserPresetSelector' in control.kind
        );
    }

    controllerTarget(control: CtrMidiProjectControl) {
        if ('CCInstrumentControllerSwitch' in control.kind) {
            const target = control.kind.CCInstrumentControllerSwitch;
            return { instrument: target.instrument, controller: target.instrument_controller };
        }
        if ('NoteInstrumentControllerToggle' in control.kind)
            return control.kind.NoteInstrumentControllerToggle;
        if ('NoteInstrumentControllerEnable' in control.kind)
            return control.kind.NoteInstrumentControllerEnable;
        if ('NoteInstrumentControllerDisable' in control.kind)
            return control.kind.NoteInstrumentControllerDisable;
        return this.firstControllerAddress();
    }

    controllerTargetValue(control: CtrMidiProjectControl) {
        const target = this.controllerTarget(control);
        return `${target.instrument}:${target.controller}`;
    }

    controllerOptionsFor(control: CtrMidiProjectControl) {
        const value = this.controllerTargetValue(control);
        return this.controllerOptions.some((option) => option.value === value)
            ? this.controllerOptions
            : [...this.controllerOptions, { value, label: `Missing controller (${value})` }];
    }

    presetIndex(control: CtrMidiProjectControl) {
        if ('NoteUserPresetSelector' in control.kind)
            return control.kind.NoteUserPresetSelector.preset_index;
        if ('CCUserPresetSelector' in control.kind)
            return control.kind.CCUserPresetSelector.preset_index;
        return 0;
    }

    presetOptionsFor(control: CtrMidiProjectControl) {
        const value = String(this.presetIndex(control));
        return this.presetOptions.some((option) => option.value === value)
            ? this.presetOptions
            : [
                  ...this.presetOptions,
                  { value, label: `Missing user preset #${Number(value) + 1}` }
              ];
    }

    noteKey(control: CtrMidiProjectControl) {
        if ('NoteInstrumentControllerToggle' in control.kind)
            return control.kind.NoteInstrumentControllerToggle.key;
        if ('NoteInstrumentControllerEnable' in control.kind)
            return control.kind.NoteInstrumentControllerEnable.key;
        if ('NoteInstrumentControllerDisable' in control.kind)
            return control.kind.NoteInstrumentControllerDisable.key;
        if ('NoteUserPresetSelector' in control.kind)
            return control.kind.NoteUserPresetSelector.key;
        return 60;
    }

    ccNumber(control: CtrMidiProjectControl) {
        if ('CCInstrumentControllerSwitch' in control.kind)
            return control.kind.CCInstrumentControllerSwitch.controller;
        if ('CCUserPresetSelector' in control.kind)
            return control.kind.CCUserPresetSelector.controller;
        return 64;
    }

    changeType(index: number, type: string) {
        const control = this.drafts[index];
        if (!control) return;
        const target = this.controllerTarget(control);
        const preset_index = this.presetIndex(control);
        switch (type as ProjectMidiControlType) {
            case 'note-controller-toggle':
                control.kind = { NoteInstrumentControllerToggle: { key: 60, ...target } };
                break;
            case 'note-controller-enable':
                control.kind = { NoteInstrumentControllerEnable: { key: 60, ...target } };
                break;
            case 'note-controller-disable':
                control.kind = { NoteInstrumentControllerDisable: { key: 60, ...target } };
                break;
            case 'cc-controller-switch':
                control.kind = {
                    CCInstrumentControllerSwitch: {
                        controller: 64,
                        threshold: 64,
                        instrument: target.instrument,
                        instrument_controller: target.controller
                    }
                };
                break;
            case 'note-preset':
                control.kind = { NoteUserPresetSelector: { key: 60, preset_index } };
                break;
            case 'cc-preset':
                control.kind = { CCUserPresetSelector: { controller: 64, preset_index } };
                break;
        }
        this.markDirty(index);
    }

    setControllerTarget(index: number, value: string) {
        const control = this.drafts[index];
        if (!control) return;
        const [instrument, controller] = value.split(':').map(Number);
        if ('NoteInstrumentControllerToggle' in control.kind)
            control.kind.NoteInstrumentControllerToggle = {
                ...control.kind.NoteInstrumentControllerToggle,
                instrument,
                controller
            };
        else if ('NoteInstrumentControllerEnable' in control.kind)
            control.kind.NoteInstrumentControllerEnable = {
                ...control.kind.NoteInstrumentControllerEnable,
                instrument,
                controller
            };
        else if ('NoteInstrumentControllerDisable' in control.kind)
            control.kind.NoteInstrumentControllerDisable = {
                ...control.kind.NoteInstrumentControllerDisable,
                instrument,
                controller
            };
        else if ('CCInstrumentControllerSwitch' in control.kind)
            control.kind.CCInstrumentControllerSwitch = {
                ...control.kind.CCInstrumentControllerSwitch,
                instrument,
                instrument_controller: controller
            };
        this.markDirty(index);
    }

    setPreset(index: number, value: string) {
        const control = this.drafts[index];
        if (!control) return;
        const preset_index = Number(value);
        if ('NoteUserPresetSelector' in control.kind)
            control.kind.NoteUserPresetSelector.preset_index = preset_index;
        else if ('CCUserPresetSelector' in control.kind)
            control.kind.CCUserPresetSelector.preset_index = preset_index;
        this.markDirty(index);
    }

    setTrigger(control: CtrMidiProjectControl, value: number) {
        if ('NoteInstrumentControllerToggle' in control.kind)
            control.kind.NoteInstrumentControllerToggle.key = value;
        else if ('NoteInstrumentControllerEnable' in control.kind)
            control.kind.NoteInstrumentControllerEnable.key = value;
        else if ('NoteInstrumentControllerDisable' in control.kind)
            control.kind.NoteInstrumentControllerDisable.key = value;
        else if ('NoteUserPresetSelector' in control.kind)
            control.kind.NoteUserPresetSelector.key = value;
        else if ('CCInstrumentControllerSwitch' in control.kind)
            control.kind.CCInstrumentControllerSwitch.controller = value;
        else if ('CCUserPresetSelector' in control.kind)
            control.kind.CCUserPresetSelector.controller = value;
    }

    markDirty(index: number) {
        this.dirty = new Set(this.dirty).add(index);
    }

    setFilterMode(index: number, key: 'source_filter' | 'channel_filter', mode: string) {
        const control = this.drafts[index];
        if (!control) return;
        control[key].type = mode as MidiFilterKind;
        this.markDirty(index);
    }

    toggleFilterValue(
        index: number,
        key: 'source_filter' | 'channel_filter',
        value: number,
        checked: boolean
    ) {
        const control = this.drafts[index];
        if (!control) return;
        const values = control[key].list;
        control[key].list = checked
            ? [...new Set([...values, value])].sort((left, right) => left - right)
            : values.filter((entry) => entry !== value);
        this.markDirty(index);
    }

    setAllFilterValues(index: number, key: 'source_filter' | 'channel_filter', values: number[]) {
        const control = this.drafts[index];
        if (!control) return;
        control[key].list = [...values];
        this.markDirty(index);
    }

    private handleLearnEvent(entry: MidiEventEntry) {
        if (this.learningIndex === null) return;
        const control = this.drafts[this.learningIndex];
        if (!control) return;
        const event = entry.event;
        if (event.type === 'NoteOn') {
            if (!this.isNoteControl(control)) {
                if (this.isControllerControl(control)) {
                    const target = this.controllerTarget(control);
                    control.kind = {
                        NoteInstrumentControllerToggle: { key: event.data.key, ...target }
                    };
                } else {
                    control.kind = {
                        NoteUserPresetSelector: {
                            key: event.data.key,
                            preset_index: this.presetIndex(control)
                        }
                    };
                }
            } else this.setTrigger(control, event.data.key);
        } else if (event.type === 'ControlChange') {
            if (this.isNoteControl(control)) {
                if (this.isControllerControl(control)) {
                    const target = this.controllerTarget(control);
                    control.kind = {
                        CCInstrumentControllerSwitch: {
                            controller: event.data.controller,
                            threshold: 64,
                            instrument: target.instrument,
                            instrument_controller: target.controller
                        }
                    };
                } else {
                    control.kind = {
                        CCUserPresetSelector: {
                            controller: event.data.controller,
                            preset_index: this.presetIndex(control)
                        }
                    };
                }
            } else this.setTrigger(control, event.data.controller);
        } else return;
        control.source_filter = { type: MidiFilterKind.Allowlist, list: [event.slot] };
        control.channel_filter = { type: MidiFilterKind.Allowlist, list: [event.channel] };
        this.markDirty(this.learningIndex);
        this.learningIndex = null;
    }

    async persist(next: CtrMidiProjectControl[], title: string) {
        if (this.saving) return false;
        this.saving = true;
        try {
            const controls: CtrMidiProjectControls = { controls: this.snapshotControls(next) };
            const saved = await saveProjectMidiAppControls(controls);
            if (saved) {
                this.drafts = this.snapshotControls(controls.controls);
                this.dirty = new Set();
            }
            return saved;
        } catch (error) {
            notify.error(errorToUserMessage(error), { title });
            return false;
        } finally {
            this.saving = false;
        }
    }

    async addControl() {
        await this.persist([...this.drafts, this.defaultControl()], 'Add project MIDI app control');
    }

    async removeControl(index: number) {
        if (this.learningIndex === index) this.learningIndex = null;
        await this.persist(
            this.drafts.filter((_, controlIndex) => controlIndex !== index),
            'Remove project MIDI app control'
        );
    }

    filterDescription(filter: MidiFilter) {
        return filter.type === MidiFilterKind.Allowlist ? 'allowed' : 'blocked';
    }
}

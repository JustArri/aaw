import { onDestroy, onMount } from 'svelte';
import {
    engineMidiControlTypeOptions,
    type EngineMidiControlType
} from '$lib/content/midi-app-controls';
import { midiChannels, midiSources } from '$lib/content/midi';
import {
    MidiFilterKind,
    type CtrMidiEngineControl,
    type CtrMidiEngineControls,
    type MidiFilter
} from '$lib/generated/models';
import { errorToUserMessage } from '$lib/error';
import { notify } from '$lib/push-notifications.svelte';
import { subscribeMidiEvents, type MidiEventEntry } from '$lib/state/midi-events.svelte';
import {
    engineMidiAppControls,
    fetchEngineMidiAppControls,
    saveEngineMidiAppControls
} from '$lib/state/midi-app-controls.svelte';

export { MidiFilterKind };

export class MidiAppControlsSection {
    readonly midiSources = midiSources;
    readonly midiChannels = midiChannels;
    readonly controlTypes = engineMidiControlTypeOptions;

    drafts = $state<CtrMidiEngineControl[]>([]);
    dirty = $state(new Set<number>());
    loading = $state(true);
    saving = $state(false);
    learningIndex = $state<number | null>(null);
    private initialized = $state(false);
    private unsubscribeMidiEvents: (() => void) | undefined;

    constructor() {
        $effect(() => {
            const controls = engineMidiAppControls().controls;
            if (this.initialized && this.dirty.size === 0 && !this.saving) {
                this.drafts = this.snapshotControls(controls);
            }
        });
        onMount(async () => {
            this.unsubscribeMidiEvents = subscribeMidiEvents((entry) =>
                this.handleLearnEvent(entry)
            );
            try {
                const controls = await fetchEngineMidiAppControls();
                this.drafts = this.snapshotControls(controls.controls);
                this.initialized = true;
            } catch (error) {
                notify.error(errorToUserMessage(error), { title: 'Load MIDI app controls' });
            } finally {
                this.loading = false;
            }
        });
        onDestroy(() => this.unsubscribeMidiEvents?.());
    }

    private defaultFilter(): MidiFilter {
        return { type: MidiFilterKind.Denylist, list: [] };
    }

    private snapshotControls(controls: CtrMidiEngineControl[]) {
        return $state.snapshot(controls);
    }

    private defaultControl(): CtrMidiEngineControl {
        return {
            name: '',
            source_filter: this.defaultFilter(),
            channel_filter: this.defaultFilter(),
            kind: { CCTransportSwitch: { controller: 64, threshold: 64 } }
        };
    }

    controlType(control: CtrMidiEngineControl): EngineMidiControlType {
        if ('NoteTransportToggle' in control.kind) return 'note-toggle';
        if ('NoteTransportEnable' in control.kind) return 'note-enable';
        if ('NoteTransportDisable' in control.kind) return 'note-disable';
        return 'cc-switch';
    }

    isNoteControl(control: CtrMidiEngineControl) {
        return !('CCTransportSwitch' in control.kind);
    }

    noteKey(control: CtrMidiEngineControl) {
        if ('NoteTransportToggle' in control.kind) return control.kind.NoteTransportToggle.key;
        if ('NoteTransportEnable' in control.kind) return control.kind.NoteTransportEnable.key;
        if ('NoteTransportDisable' in control.kind) return control.kind.NoteTransportDisable.key;
        return 60;
    }

    setNoteKey(control: CtrMidiEngineControl, key: number) {
        if ('NoteTransportEnable' in control.kind) {
            control.kind = { NoteTransportEnable: { key } };
        } else if ('NoteTransportDisable' in control.kind) {
            control.kind = { NoteTransportDisable: { key } };
        } else {
            control.kind = { NoteTransportToggle: { key } };
        }
    }

    markDirty(index: number) {
        this.dirty = new Set(this.dirty).add(index);
    }

    changeType(index: number, type: string) {
        const control = this.drafts[index];
        if (!control) return;
        switch (type as EngineMidiControlType) {
            case 'note-toggle':
                control.kind = { NoteTransportToggle: { key: 60 } };
                break;
            case 'note-enable':
                control.kind = { NoteTransportEnable: { key: 60 } };
                break;
            case 'note-disable':
                control.kind = { NoteTransportDisable: { key: 60 } };
                break;
            case 'cc-switch':
                control.kind = { CCTransportSwitch: { controller: 64, threshold: 64 } };
                break;
        }
        this.markDirty(index);
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
        if (!control) {
            this.learningIndex = null;
            return;
        }
        const event = entry.event;
        if (event.type === 'NoteOn') {
            this.setNoteKey(control, event.data.key);
        } else if (event.type === 'ControlChange') {
            control.kind = {
                CCTransportSwitch: { controller: event.data.controller, threshold: 64 }
            };
        } else return;
        control.source_filter = { type: MidiFilterKind.Allowlist, list: [event.slot] };
        control.channel_filter = { type: MidiFilterKind.Allowlist, list: [event.channel] };
        this.markDirty(this.learningIndex);
        this.learningIndex = null;
    }

    private async persist(next: CtrMidiEngineControl[], title: string) {
        if (this.saving) return false;
        this.saving = true;
        try {
            const controls: CtrMidiEngineControls = { controls: this.snapshotControls(next) };
            const saved = await saveEngineMidiAppControls(controls);
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
        await this.persist([...this.drafts, this.defaultControl()], 'Add MIDI app control');
    }

    async applyControl() {
        await this.persist(this.drafts, 'Update MIDI app control');
    }

    async removeControl(index: number) {
        this.learningIndex = this.learningIndex === index ? null : this.learningIndex;
        await this.persist(
            this.drafts.filter((_, controlIndex) => controlIndex !== index),
            'Remove MIDI app control'
        );
    }

    filterDescription(filter: MidiFilter) {
        return filter.type === MidiFilterKind.Allowlist ? 'allowed' : 'blocked';
    }
}

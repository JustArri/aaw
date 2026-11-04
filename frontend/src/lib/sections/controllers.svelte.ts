import { page } from '$app/state';
import { onDestroy, onMount, tick } from 'svelte';
import * as api from '$lib/generated/api';
import type {
    DrumLane,
    DrumLaneSequence,
    DrumMachineController,
    DrumStep
} from '$lib/generated/models';
import { errorToUserMessage } from '$lib/error';
import { notify } from '$lib/push-notifications.svelte';
import { activeProject } from '$lib/state/active-project.svelte';
import { drumVoiceNames, beatPresets, lanePresets } from '$lib/content/drum-machine';
import { TransportClock } from '$lib/transport-clock.svelte';

export type Machine = { id: number; data: DrumMachineController; savedName: string };
type SelectedStep = { machineId: number; laneId: number; index: number };
type SelectedVoice = {
    machineId: number;
    laneId: number;
    initialName: string;
    initialNote: number;
    initialChannel: number;
    initialVelocity: number;
};

export class ControllersSection {
    readonly lanePresets = lanePresets;
    readonly beatPresets = beatPresets;
    requestedInstrument = $derived(page.url.searchParams.get('instrument'));
    instrumentId = $derived(
        this.requestedInstrument === null ? Number.NaN : Number(this.requestedInstrument)
    );
    machines = $state<Machine[]>([]);
    loading = $state(true);
    adding = $state(false);
    addingMidiController = $state(false);
    controllerEditorRevision = $state(0);
    saving = $state(new Set<number>());
    dirtySaves = $state(new Set<number>());
    removing = $state(new Set<number>());
    selectedLanePresets = $state<Record<number, string>>({});
    selectedBeatPresets = $state<Record<number, string>>({});
    selectedStep = $state<SelectedStep | null>(null);
    selectedVoice = $state<SelectedVoice | null>(null);
    voiceEditor = $state<HTMLDivElement>();
    voiceEditorHighlighted = $state(false);
    readonly transportClock = new TransportClock();
    meterBeatSteps = $derived(16 / (activeProject()?.transport.beat_unit ?? 4));
    previewQuantum = $derived(
        Math.max(
            1,
            Math.min(
                ...this.machines.flatMap(({ data }) =>
                    data.sequences.map(({ step_resolution_ticks }) => step_resolution_ticks)
                ),
                960
            )
        )
    );
    previewPlaying = $derived(this.transportClock.synchronized);
    previewTick = $derived(
        Math.floor(this.transportClock.tick / this.previewQuantum) * this.previewQuantum
    );

    private voiceHighlightTimeout: ReturnType<typeof setTimeout> | undefined;
    constructor() {
        onMount(() => {
            void this.load();
        });
        onDestroy(() => {
            clearTimeout(this.voiceHighlightTimeout);
        });
    }

    previewed(sequence: DrumLaneSequence | undefined, index: number) {
        if (!this.previewPlaying || !sequence || sequence.length_steps === 0) return false;
        return (
            Math.floor(this.previewTick / sequence.step_resolution_ticks) %
                sequence.length_steps ===
            index
        );
    }

    async load() {
        if (!Number.isInteger(this.instrumentId) || this.instrumentId < 0) {
            this.loading = false;
            return;
        }
        this.loading = true;
        try {
            const controllers = await api.controllerList(this.instrumentId);
            this.machines = controllers.data.flatMap(([id, controller]) =>
                controller.type === 'DrumMachine'
                    ? [{ id, data: controller.data, savedName: controller.data.name }]
                    : []
            );
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Load drum machines' });
        } finally {
            this.loading = false;
        }
    }

    async addMachine() {
        this.adding = true;
        try {
            await api.drumMachineAppend(this.instrumentId);
            await this.load();
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Add drum machine' });
        } finally {
            this.adding = false;
        }
    }

    async addMidiController() {
        this.addingMidiController = true;
        try {
            await api.controllerAppend(this.instrumentId);
            this.controllerEditorRevision += 1;
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Add MIDI input controller' });
        } finally {
            this.addingMidiController = false;
        }
    }

    async saveMachine(machine: Machine): Promise<boolean> {
        if (this.saving.has(machine.id)) {
            this.dirtySaves = new Set(this.dirtySaves).add(machine.id);
            return true;
        }
        this.saving = new Set(this.saving).add(machine.id);
        try {
            do {
                const nextDirty = new Set(this.dirtySaves);
                nextDirty.delete(machine.id);
                this.dirtySaves = nextDirty;
                const savedName = machine.data.name;
                const response = await api.controllerReplace(this.instrumentId, machine.id, {
                    type: 'DrumMachine',
                    data: machine.data
                });
                if (response.status !== 204) throw new Error(`Update failed (${response.status})`);
                machine.savedName = savedName;
            } while (this.dirtySaves.has(machine.id));
            return true;
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Update drum machine' });
            await this.load();
            return false;
        } finally {
            const next = new Set(this.saving);
            next.delete(machine.id);
            this.saving = next;
        }
    }

    async removeMachine(machine: Machine) {
        this.removing = new Set(this.removing).add(machine.id);
        try {
            await api.controllerRemove(this.instrumentId, machine.id);
            this.machines = this.machines.filter(({ id }) => id !== machine.id);
        } catch (cause) {
            notify.error(errorToUserMessage(cause), { title: 'Remove drum machine' });
        } finally {
            const next = new Set(this.removing);
            next.delete(machine.id);
            this.removing = next;
        }
    }

    private barLayout(machine: Machine) {
        const resolution = machine.data.sequences[0]?.step_resolution_ticks ?? 240;
        const transport = activeProject()?.transport;
        const barTicks = transport
            ? ((960 * 4) / transport.beat_unit) * transport.beats_per_bar
            : 3840;
        return {
            resolution,
            barTicks,
            lengthSteps: Math.max(1, Math.floor(barTicks / resolution))
        };
    }

    private emptySteps(lengthSteps: number, resolution: number) {
        return Array.from({ length: lengthSteps }, () => ({
            enabled: false,
            velocity: 100,
            micro_offset_ticks: 0,
            gate_length_ticks: Math.max(1, Math.floor(resolution / 2))
        }));
    }

    async addLane(machine: Machine) {
        if (machine.data.lanes.length >= 64) return;
        const usedIds = new Set(machine.data.lanes.map(({ id }) => id));
        let id = 0;
        while (usedIds.has(id)) id += 1;
        const { resolution, lengthSteps } = this.barLayout(machine);
        const lane: DrumLane = {
            id,
            name: `Lane ${machine.data.lanes.length + 1}`,
            output_note: 36,
            output_channel: 9,
            velocity: 127,
            gate_mode: 'OneShot',
            choke_group: null,
            muted: false,
            solo: false
        };
        machine.data.lanes.push(lane);
        machine.data.sequences.push({
            lane_id: id,
            length_steps: lengthSteps,
            step_resolution_ticks: resolution,
            steps: this.emptySteps(lengthSteps, resolution)
        });
        if (await this.saveMachine(machine)) await this.selectVoice(machine, lane);
    }

    async removeLane(machine: Machine, lane: DrumLane) {
        machine.data.lanes = machine.data.lanes.filter(({ id }) => id !== lane.id);
        machine.data.sequences = machine.data.sequences.filter(
            ({ lane_id }) => lane_id !== lane.id
        );
        if (this.selectedVoice?.machineId === machine.id && this.selectedVoice.laneId === lane.id)
            this.selectedVoice = null;
        if (this.selectedStep?.machineId === machine.id && this.selectedStep.laneId === lane.id)
            this.selectedStep = null;
        await this.saveMachine(machine);
    }

    async removeAllLanes(machine: Machine) {
        machine.data.lanes = [];
        machine.data.sequences = [];
        if (this.selectedVoice?.machineId === machine.id) this.selectedVoice = null;
        if (this.selectedStep?.machineId === machine.id) this.selectedStep = null;
        await this.saveMachine(machine);
    }

    async applyLanePreset(machine: Machine) {
        const preset = lanePresets.find(
            ({ value }) => value === this.selectedLanePresets[machine.id]
        );
        if (!preset) return;
        const { resolution, lengthSteps } = this.barLayout(machine);
        machine.data.lanes = preset.lanes.map(([name, output_note], id) => ({
            id,
            name,
            output_note,
            output_channel: 9,
            velocity: 127,
            gate_mode: 'OneShot',
            choke_group: null,
            muted: false,
            solo: false
        }));
        machine.data.sequences = machine.data.lanes.map(({ id }) => ({
            lane_id: id,
            length_steps: lengthSteps,
            step_resolution_ticks: resolution,
            steps: this.emptySteps(lengthSteps, resolution)
        }));
        if (this.selectedVoice?.machineId === machine.id) this.selectedVoice = null;
        if (this.selectedStep?.machineId === machine.id) this.selectedStep = null;
        if (await this.saveMachine(machine)) {
            this.selectedLanePresets = { ...this.selectedLanePresets, [machine.id]: '' };
        }
    }

    async applyBeatPreset(machine: Machine) {
        const preset = beatPresets.find(
            ({ value }) => value === this.selectedBeatPresets[machine.id]
        );
        if (!preset) return;
        const { resolution, barTicks, lengthSteps } = this.barLayout(machine);
        machine.data.lanes = preset.voices.map((voice, id) => {
            const output_note =
                voice.preferredNote ?? (voice.notes.includes(36) ? 36 : voice.notes[0]);
            return {
                id,
                name: drumVoiceNames[output_note] ?? `Note ${output_note}`,
                output_note,
                output_channel: 9,
                velocity: 127,
                gate_mode: 'OneShot' as const,
                choke_group: null,
                muted: false,
                solo: false
            };
        });
        machine.data.sequences = machine.data.lanes.map(({ id }) => ({
            lane_id: id,
            length_steps: lengthSteps,
            step_resolution_ticks: resolution,
            steps: this.emptySteps(lengthSteps, resolution)
        }));
        const presetCycleTicks = 960 * 4;
        const presetStepTicks = 960 / 4;
        for (const [voiceIndex, voice] of preset.voices.entries()) {
            const laneSequence = machine.data.sequences[voiceIndex];
            for (
                let cycleStartTick = 0;
                cycleStartTick < barTicks;
                cycleStartTick += presetCycleTicks
            ) {
                for (const [presetStep, velocity] of voice.hits) {
                    const hitTick = cycleStartTick + presetStep * presetStepTicks;
                    if (hitTick >= barTicks) continue;
                    const index = Math.round(hitTick / resolution);
                    const step = laneSequence.steps[index];
                    if (step && index < laneSequence.length_steps) {
                        step.velocity = step.enabled ? Math.max(step.velocity, velocity) : velocity;
                        step.enabled = true;
                    }
                }
            }
        }
        this.selectedStep = null;
        if (this.selectedVoice?.machineId === machine.id) this.selectedVoice = null;
        if (await this.saveMachine(machine)) {
            this.selectedBeatPresets = { ...this.selectedBeatPresets, [machine.id]: '' };
            this.selectedLanePresets = { ...this.selectedLanePresets, [machine.id]: '' };
        }
    }

    sequence(machine: Machine, lane: DrumLane) {
        return machine.data.sequences.find(({ lane_id }) => lane_id === lane.id);
    }

    selected(machine: Machine, lane: DrumLane, index: number) {
        return (
            this.selectedStep?.machineId === machine.id &&
            this.selectedStep.laneId === lane.id &&
            this.selectedStep.index === index
        );
    }

    selectedStepValue(machine: Machine): DrumStep | undefined {
        if (this.selectedStep?.machineId !== machine.id) return;
        const lane = machine.data.lanes.find(({ id }) => id === this.selectedStep?.laneId);
        return lane ? this.sequence(machine, lane)?.steps[this.selectedStep.index] : undefined;
    }

    selectedLane(machine: Machine): DrumLane | undefined {
        if (this.selectedStep?.machineId !== machine.id) return;
        return machine.data.lanes.find(({ id }) => id === this.selectedStep?.laneId);
    }

    selectedVoiceLane(machine: Machine): DrumLane | undefined {
        if (this.selectedVoice?.machineId !== machine.id) return;
        return machine.data.lanes.find(({ id }) => id === this.selectedVoice?.laneId);
    }

    async selectVoice(machine: Machine, lane: DrumLane) {
        clearTimeout(this.voiceHighlightTimeout);
        this.voiceEditorHighlighted = false;
        this.selectedVoice = {
            machineId: machine.id,
            laneId: lane.id,
            initialName: lane.name,
            initialNote: lane.output_note,
            initialChannel: lane.output_channel + 1,
            initialVelocity: lane.velocity
        };
        await tick();
        this.voiceEditor?.scrollIntoView({
            behavior: 'smooth',
            block: 'nearest',
            inline: 'nearest'
        });
        this.voiceEditor?.querySelector<HTMLInputElement>('input')?.focus({ preventScroll: true });
        this.voiceEditorHighlighted = true;
        this.voiceHighlightTimeout = setTimeout(() => {
            this.voiceEditorHighlighted = false;
        }, 900);
    }

    async toggleStep(machine: Machine, lane: DrumLane, index: number) {
        const step = this.sequence(machine, lane)?.steps[index];
        if (!step) return;
        step.enabled = !step.enabled;
        this.selectedStep = { machineId: machine.id, laneId: lane.id, index };
        await this.saveMachine(machine);
    }

    laneHasSteps(machine: Machine, lane: DrumLane) {
        return this.sequence(machine, lane)?.steps.some(({ enabled }) => enabled) ?? false;
    }

    machineHasSteps(machine: Machine) {
        return machine.data.sequences.some((laneSequence) =>
            laneSequence.steps.some(({ enabled }) => enabled)
        );
    }

    async clearLane(machine: Machine, lane: DrumLane) {
        const laneSequence = this.sequence(machine, lane);
        if (!laneSequence) return;
        for (const step of laneSequence.steps) step.enabled = false;
        await this.saveMachine(machine);
    }

    async clearMachine(machine: Machine) {
        for (const laneSequence of machine.data.sequences) {
            for (const step of laneSequence.steps) step.enabled = false;
        }
        await this.saveMachine(machine);
    }
}

import { gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type {
    CtrInstrumentKind,
    Effects,
    Instrument,
    InstrumentPushRequest,
    ParameterInfo,
    SfzInfo,
    SoundFontInfo,
    SoundFontProgram
} from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

export const SynthKind = { Dummy: 'Dummy', Fluid: 'Fluid', Sfizz: 'Sfizz', Clap: 'Clap' } as const;
export type SynthKind = (typeof SynthKind)[keyof typeof SynthKind];
export type SynthPushParams = InstrumentPushRequest;

export type ControllerSlot = {
    instrument_id: number;
    controller_id: number;
    synth_id: number;
    enabled: boolean;
    ignore_key_transform: boolean;
    key_transform: import('$lib/generated/models').MidiKeyTransform;
    ignore_velocity_transform: boolean;
    velocity_transform: import('$lib/generated/models').MidiVelocityTransform;
    channel_filter: import('$lib/generated/models').MidiFilter;
    cc_filter: import('$lib/generated/models').MidiFilter;
    ignore_pitch_bend: boolean;
    ignore_program_change: boolean;
    ignore_channel_aftertouch: boolean;
    ignore_polyphonic_aftertouch: boolean;
    use_global_transposition: boolean;
    name: string;
};

type DummySynth = { name: string; max_voices: number };
type FluidSynth = {
    name: string;
    file_path: string;
    info: SoundFontInfo;
    max_voices: number;
    program: SoundFontProgram;
};
type SfizzSynth = {
    name: string;
    file_path: string;
    info: SfzInfo;
    max_voices: number;
    preload_size: number;
};
type ClapSynth = {
    name: string;
    file_path: string;
    plugin_id: string;
    plugin_name: string | null;
    plugin_version: string | null;
    plugin_parameters: ParameterInfo[];
};

export type SynthEntry =
    | { id: number; kind: 'Dummy'; synth: DummySynth; effects: Effects; gain: number }
    | { id: number; kind: 'Fluid'; synth: FluidSynth; effects: Effects; gain: number }
    | { id: number; kind: 'Sfizz'; synth: SfizzSynth; effects: Effects; gain: number }
    | { id: number; kind: 'Clap'; synth: ClapSynth; effects: Effects; gain: number };

let instrumentsState = $state({} as Record<string, Instrument>);
let fetchPromise: Promise<void> | null = null;

export function instrumentSlots(): Record<string, ControllerSlot> {
    return Object.fromEntries(
        Object.entries(instrumentsState).flatMap(([instrumentId, instrument]) => {
            return Object.entries(instrument.controllers).flatMap(([controllerId, controller]) => {
                if (controller.type !== 'MidiInput') {
                    return [];
                }
                const key = Number(instrumentId) * 65_536 + Number(controllerId);
                return [
                    [
                        String(key),
                        {
                            ...controller.data,
                            instrument_id: Number(instrumentId),
                            controller_id: Number(controllerId),
                            synth_id: Number(instrumentId)
                        }
                    ]
                ];
            });
        })
    );
}

function toEntry(id: string, instrument: Instrument): SynthEntry {
    const common = { id: Number(id), effects: instrument.effects, gain: instrument.gain };
    if ('Dummy' in instrument.kind) {
        return {
            ...common,
            kind: SynthKind.Dummy,
            synth: { name: instrument.name, ...instrument.kind.Dummy }
        };
    }
    if ('Fluid' in instrument.kind) {
        return {
            ...common,
            kind: SynthKind.Fluid,
            synth: { name: instrument.name, ...instrument.kind.Fluid }
        };
    }
    if ('Sfizz' in instrument.kind) {
        return {
            ...common,
            kind: SynthKind.Sfizz,
            synth: { name: instrument.name, ...instrument.kind.Sfizz }
        };
    }
    return {
        ...common,
        kind: SynthKind.Clap,
        synth: { name: instrument.name, ...instrument.kind.Clap }
    };
}

export function synths(): SynthEntry[] {
    return Object.entries(instrumentsState).map(([id, instrument]) => toEntry(id, instrument));
}

export function resetSynths() {
    instrumentsState = {};
}

function updateInstrument(id: number, update: Partial<Instrument>) {
    const instrument = instrumentsState[String(id)];
    if (instrument) {
        instrumentsState = { ...instrumentsState, [id]: { ...instrument, ...update } };
    }
}

export function updateSynthName(id: number, name: string) {
    updateInstrument(id, { name });
}

export function updateSynthParameters(id: number, kind: CtrInstrumentKind) {
    updateInstrument(id, { kind });
}

export function updateSynthEffects(id: number, effects: Effects) {
    updateInstrument(id, { effects });
}

export function updateSynthGain(id: number, gain: number) {
    updateInstrument(id, { gain });
}

export function updateClapParameter(id: number, parameterId: number, value: number) {
    const instrument = instrumentsState[String(id)];
    if (!instrument || !('Clap' in instrument.kind)) {
        return;
    }
    updateInstrument(id, {
        kind: {
            Clap: {
                ...instrument.kind.Clap,
                plugin_parameters: instrument.kind.Clap.plugin_parameters.map((parameter) =>
                    parameter.id === parameterId
                        ? { ...parameter, value, value_text: null }
                        : parameter
                )
            }
        }
    });
}

export function removeSynthFromState(id: number) {
    const next = { ...instrumentsState };
    delete next[String(id)];
    instrumentsState = next;
}

export function updateFluidSynthProgram(id: number, program: SoundFontProgram) {
    const instrument = instrumentsState[String(id)];
    if (instrument && 'Fluid' in instrument.kind) {
        updateInstrument(id, { kind: { Fluid: { ...instrument.kind.Fluid, program } } });
    }
}

export function updateSynthInstrumentSlot(id: number, update: Partial<ControllerSlot>) {
    const instrumentId = Math.floor(id / 65_536);
    const requestedControllerId = id % 65_536;
    const instrument = instrumentsState[String(instrumentId)];
    if (!instrument) {
        return;
    }
    const entry = Object.entries(instrument.controllers).find(
        ([controllerId]) => Number(controllerId) === requestedControllerId
    );
    if (!entry) {
        return;
    }
    const [controllerId, controller] = entry as [
        string,
        import('$lib/generated/models').CtrController
    ];
    if (controller.type === 'MidiInput') {
        updateInstrument(instrumentId, {
            controllers: {
                ...instrument.controllers,
                [controllerId]: {
                    ...controller,
                    data: {
                        ...controller.data,
                        ...Object.fromEntries(
                            Object.entries(update).filter(
                                ([key]) =>
                                    key !== 'instrument_id' &&
                                    key !== 'controller_id' &&
                                    key !== 'synth_id'
                            )
                        )
                    }
                }
            }
        });
    }
}

export async function fetchSynths(forceFresh = false) {
    if (fetchPromise) {
        await fetchPromise;
        if (!forceFresh) {
            return;
        }
    }
    const currentFetch = (async () => {
        try {
            instrumentsState = (await api.instrumentList()).data;
        } catch (error) {
            resetSynths();
            reportGatewayOffline();
            throw gatewayNotResponding(error);
        }
    })();
    fetchPromise = currentFetch;
    try {
        await currentFetch;
    } finally {
        if (fetchPromise === currentFetch) {
            fetchPromise = null;
        }
    }
}

export async function fetchSynthInstrumentSlots() {
    await fetchSynths(true);
}

function responseError(status: number, data: unknown): never {
    if (status === 500) {
        reportGatewayError(status);
    }
    if (typeof data === 'string' && data.trim()) {
        throw new Error(data);
    }
    throw new Error(`Instrument operation failed (${status})`);
}

async function request<T>(operation: () => Promise<T>): Promise<T> {
    try {
        return await operation();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }
}

export async function addSynth(details: InstrumentPushRequest) {
    const response = await request(() => api.instrumentPush(details));
    if (response.status !== 200) {
        responseError(response.status, response.data);
    }
    await fetchSynths(true);
    return response.data;
}

export async function replaceSynth(id: number, details: InstrumentPushRequest) {
    const response = await request(() => api.instrumentReplace(id, details));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    await fetchSynths(true);
}

export async function removeSynth(id: number) {
    const response = await request(() => api.instrumentRemove(id));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    removeSynthFromState(id);
}

export async function setSynthName(id: number, name: string) {
    const response = await request(() => api.instrumentSetName(id, name));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateSynthName(id, name);
}

export async function setFluidProgram(id: number, program: SoundFontProgram) {
    const response = await request(() => api.instrumentSetSoundfontProgram(id, program));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateFluidSynthProgram(id, program);
}

export async function setGlobalGain(gain: number) {
    const response = await request(() => api.setGlobalGain(gain));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
}

export async function setSynthGain(id: number, gain: number) {
    const response = await request(() => api.instrumentSetGain(id, gain));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
}

export async function setClapParameter(id: number, parameterId: number, value: number) {
    const response = await request(() => api.instrumentSetClapParameter(id, parameterId, value));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateClapParameter(id, parameterId, value);
}

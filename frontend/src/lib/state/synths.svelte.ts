import { gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type {
    Effects,
    CtrSynthParameters,
    Instrument,
    SfzInfo,
    SoundFontInfo,
    SoundFontProgram,
    Synth,
    SynthPushRequest
} from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

export const SynthKind = { Dummy: 'Dummy', Fluid: 'Fluid', Sfizz: 'Sfizz' } as const;
export type SynthKind = (typeof SynthKind)[keyof typeof SynthKind];

type DummySynth = { name: string };
type FluidSynth = {
    name: string;
    file_path: string;
    info: SoundFontInfo;
    program: SoundFontProgram;
};
type SfizzSynth = { name: string; file_path: string; info: SfzInfo };

export type SynthEntry =
    | { id: number; kind: typeof SynthKind.Dummy; synth: DummySynth; effects: Effects }
    | { id: number; kind: typeof SynthKind.Fluid; synth: FluidSynth; effects: Effects }
    | { id: number; kind: typeof SynthKind.Sfizz; synth: SfizzSynth; effects: Effects };

let synthsState = $state({} as Record<string, Synth>);
let instrumentsState = $state({} as Record<string, Instrument>);
let fetching = false;

export function instrumentSlots() {
    return instrumentsState;
}

function toEntry(id: string, synth: Synth): SynthEntry {
    if (synth.parameters === SynthKind.Dummy) {
        return {
            id: Number(id),
            kind: SynthKind.Dummy,
            synth: { name: synth.name },
            effects: synth.effects
        };
    }
    if ('Fluid' in synth.parameters) {
        return {
            id: Number(id),
            kind: SynthKind.Fluid,
            synth: { name: synth.name, ...synth.parameters.Fluid },
            effects: synth.effects
        };
    }
    return {
        id: Number(id),
        kind: SynthKind.Sfizz,
        synth: { name: synth.name, ...synth.parameters.Sfizz },
        effects: synth.effects
    };
}

export function synths(): SynthEntry[] {
    return Object.entries(synthsState).map(([id, synth]) => toEntry(id, synth));
}

export function resetSynths() {
    synthsState = {};
    instrumentsState = {};
}

export function updateSynthName(id: number, name: string) {
    const synth = synthsState[String(id)];
    if (synth) {
        synthsState = { ...synthsState, [id]: { ...synth, name } };
    }
}

export function updateSynthParameters(id: number, parameters: CtrSynthParameters) {
    const synth = synthsState[String(id)];
    if (synth) {
        synthsState = { ...synthsState, [id]: { ...synth, parameters } };
    }
}

export function updateSynthEffects(id: number, effects: Effects) {
    const synth = synthsState[String(id)];
    if (synth) {
        synthsState = { ...synthsState, [id]: { ...synth, effects } };
    }
}

export function removeSynthFromState(id: number) {
    const next = { ...synthsState };
    delete next[String(id)];
    synthsState = next;
}

export function updateFluidSynthProgram(id: number, program: SoundFontProgram) {
    const synth = synthsState[String(id)];
    if (!synth || synth.parameters === SynthKind.Dummy || !('Fluid' in synth.parameters)) {
        return;
    }
    synthsState = {
        ...synthsState,
        [id]: {
            ...synth,
            parameters: { Fluid: { ...synth.parameters.Fluid, program } }
        }
    };
}

export function updateSynthInstrumentSlot(id: number, update: Partial<Instrument>) {
    const instrument = instrumentsState[String(id)];
    if (instrument) {
        instrumentsState = { ...instrumentsState, [id]: { ...instrument, ...update } };
    }
}

export async function fetchSynths() {
    if (fetching) {
        return;
    }
    fetching = true;
    try {
        const [synthResponse, instrumentResponse] = await Promise.all([
            api.synthList(),
            api.instrumentList()
        ]);
        synthsState = synthResponse.data;
        instrumentsState = instrumentResponse.data;
    } catch (error) {
        resetSynths();
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetching = false;
    }
}

export async function fetchSynthInstrumentSlots() {
    try {
        const response = await api.instrumentList();
        instrumentsState = response.data;
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }
}

function responseError(status: number, data: unknown): never {
    if (status === 500) {
        reportGatewayError(status);
    }
    if (typeof data === 'string' && data.trim()) {
        throw new Error(data);
    }
    if (status === 404) {
        throw new Error('Synth does not exist');
    }
    throw new Error(`Synth operation failed (${status})`);
}

async function request<T>(operation: () => Promise<T>): Promise<T> {
    try {
        return await operation();
    } catch (error) {
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    }
}

function pushRequest(kind: SynthKind, path?: string): SynthPushRequest {
    if (kind === SynthKind.Dummy) {
        return { type: SynthKind.Dummy };
    }
    return { type: kind, data: { path: path ?? '' } };
}

export async function addSynth(kind: SynthKind, path?: string) {
    const response = await request(() => api.synthPush(pushRequest(kind, path)));
    if (response.status !== 200) {
        responseError(response.status, response.data);
    }
    await fetchSynths();
    return response.data;
}

export async function replaceSynth(kind: SynthKind, id: number, path?: string) {
    const response = await request(() => api.synthReplace(id, pushRequest(kind, path)));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    await fetchSynths();
}

export async function removeSynth(id: number) {
    const response = await request(() => api.synthRemove(id));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    removeSynthFromState(id);
}

export async function setSynthName(id: number, name: string) {
    const response = await request(() => api.synthSetName(id, name));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateSynthName(id, name);
}

export async function setFluidProgram(id: number, program: SoundFontProgram) {
    const response = await request(() => api.synthSetSoundfontProgram(id, program));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateFluidSynthProgram(id, program);
}

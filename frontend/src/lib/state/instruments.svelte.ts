import { gatewayNotResponding } from '$lib/error';
import * as api from '$lib/generated/api';
import type {
    Instrument,
    MidiFilter,
    MidiKeyTransform,
    MidiVelocityTransform,
    SynthId
} from '$lib/generated/models';
import { reportGatewayError, reportGatewayOffline } from './gateway.svelte';

let slotsState = $state({} as Record<string, Instrument>);
let globalTranspositionState = $state(0);
let fetching = false;

export function instrumentSlots() {
    return slotsState;
}

export function instrumentGlobalTransposition() {
    return globalTranspositionState;
}

export function resetInstruments() {
    slotsState = {};
    globalTranspositionState = 0;
}

export async function fetchInstruments() {
    if (fetching) {
        return;
    }
    fetching = true;

    try {
        const [slotsResponse, transpositionResponse] = await Promise.all([
            api.instrumentList(),
            api.globalTransposition()
        ]);
        slotsState = slotsResponse.data;
        globalTranspositionState = transpositionResponse.data;
    } catch (error) {
        resetInstruments();
        reportGatewayOffline();
        throw gatewayNotResponding(error);
    } finally {
        fetching = false;
    }
}

export async function fetchInstrumentSlots() {
    const response = await request(api.instrumentList);
    slotsState = response.data;
}

function responseError(status: number, data: unknown): never {
    if (status === 500) {
        reportGatewayError(status);
    }
    if (typeof data === 'string' && data.trim()) {
        throw new Error(data);
    }
    if (status === 404) {
        throw new Error('Instrument does not exist');
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

export function updateInstrumentSlot(id: number, update: Partial<Instrument>) {
    const slot = slotsState[String(id)];
    if (slot) {
        slotsState = { ...slotsState, [id]: { ...slot, ...update } };
    }
}

export function updateInstrumentGlobalTransposition(value: number) {
    globalTranspositionState = value;
}

export async function addInstrument() {
    const response = await request(api.instrumentAppendNew);
    if (response.status !== 200) {
        responseError(response.status, response.data);
    }
    await fetchInstrumentSlots();
    return response.data;
}

export async function removeInstrument(id: number) {
    const response = await request(() => api.instrumentRemove(id));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    const next = { ...slotsState };
    delete next[String(id)];
    slotsState = next;
}

export async function setInstrumentEnabled(id: number, enabled: boolean) {
    const response = await request(() => api.instrumentSetEnabled(id, enabled));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { enabled });
}

export async function setInstrumentSynth(id: number, synthId: SynthId | null) {
    const response = await request(() => api.instrumentSetSynth(id, synthId));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { synth_id: synthId });
}

export async function setInstrumentKeyTransform(id: number, transform: MidiKeyTransform) {
    const response = await request(() => api.instrumentSetKeyTransform(id, transform));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { key_transform: transform });
}

export async function setInstrumentVelocityTransform(id: number, transform: MidiVelocityTransform) {
    const response = await request(() => api.instrumentSetVelocityTransform(id, transform));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { velocity_transform: transform });
}

export async function setInstrumentChannelFilter(id: number, filter: MidiFilter) {
    const response = await request(() => api.instrumentSetChannelFilter(id, filter));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { channel_filter: filter });
}

export async function setInstrumentCcFilter(id: number, filter: MidiFilter) {
    const response = await request(() => api.instrumentSetCcFilter(id, filter));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { cc_filter: filter });
}

export async function setInstrumentIgnoreKeyTransform(id: number, enabled: boolean) {
    const response = await request(() => api.instrumentSetIgnoreKeyTransform(id, enabled));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { ignore_key_transform: enabled });
}

export async function setInstrumentIgnoreVelocityTransform(id: number, enabled: boolean) {
    const response = await request(() => api.instrumentSetIgnoreVelocityTransform(id, enabled));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { ignore_velocity_transform: enabled });
}

export async function setInstrumentIgnorePitchBend(id: number, enabled: boolean) {
    const response = await request(() => api.instrumentSetIgnorePitchBend(id, enabled));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { ignore_pitch_bend: enabled });
}

export async function setInstrumentIgnoreProgramChange(id: number, enabled: boolean) {
    const response = await request(() => api.instrumentSetIgnoreProgramChange(id, enabled));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { ignore_program_change: enabled });
}

export async function setInstrumentIgnoreChannelAftertouch(id: number, enabled: boolean) {
    const response = await request(() => api.instrumentSetIgnoreChannelAftertouch(id, enabled));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { ignore_channel_aftertouch: enabled });
}

export async function setInstrumentIgnorePolyphonicAftertouch(id: number, enabled: boolean) {
    const response = await request(() => api.instrumentSetIgnorePolyphonicAftertouch(id, enabled));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { ignore_polyphonic_aftertouch: enabled });
}

export async function setInstrumentUsesGlobalTransposition(id: number, enabled: boolean) {
    const response = await request(() => api.instrumentSetUseGlobalTransposition(id, enabled));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { use_global_transposition: enabled });
}

export async function fetchInstrumentName(id: number) {
    const response = await request(() => api.instrumentName(id));
    if (response.status !== 200) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { name: response.data });
}

export async function setInstrumentName(id: number, name: string) {
    const response = await request(() => api.instrumentSetName(id, name));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentSlot(id, { name });
}

export async function setInstrumentGlobalTransposition(value: number) {
    const response = await request(() => api.setGlobalTransposition(value));
    if (response.status !== 204) {
        responseError(response.status, response.data);
    }
    updateInstrumentGlobalTransposition(value);
    return value;
}
